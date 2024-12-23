// SPDX-License-Identifier: GPL-2.0

// Copyright (C) 2024 Google LLC.

//! Miscdevice support.
//!
//! C headers: [`include/linux/miscdevice.h`](srctree/include/linux/miscdevice.h).
//!
//! Reference: <https://www.kernel.org/doc/html/latest/driver-api/misc_devices.html>

use crate::{
    bindings,
    device::Device,
    error::{to_result, Error, Result, VTABLE_DEFAULT_ERROR},
    ffi::{c_int, c_long, c_uint, c_ulong},
    fs::File,
    prelude::*,
    seq_file::SeqFile,
    str::CStr,
    types::{ForeignOwnable, Opaque},
};
use core::{marker::PhantomData, mem::MaybeUninit, pin::Pin, ptr::NonNull};

/// The kernel `loff_t` type.
#[allow(non_camel_case_types)]
pub type loff_t = bindings::loff_t;

/// Options for creating a misc device.
#[derive(Copy, Clone)]
pub struct MiscDeviceOptions {
    /// The name of the miscdevice.
    pub name: &'static CStr,
}

impl MiscDeviceOptions {
    /// Create a raw `struct miscdev` ready for registration.
    pub const fn into_raw<T: MiscDevice>(self) -> bindings::miscdevice {
        // SAFETY: All zeros is valid for this C type.
        let mut result: bindings::miscdevice = unsafe { MaybeUninit::zeroed().assume_init() };
        result.minor = bindings::MISC_DYNAMIC_MINOR as _;
        result.name = self.name.as_char_ptr();
        result.fops = create_vtable::<T>();
        result
    }
}

/// A registration of a miscdevice.
///
/// # Invariants
///
/// `inner` is a registered misc device.
#[repr(transparent)]
#[pin_data(PinnedDrop)]
pub struct MiscDeviceRegistration<T> {
    #[pin]
    inner: Opaque<bindings::miscdevice>,
    _t: PhantomData<T>,
}

// SAFETY: It is allowed to call `misc_deregister` on a different thread from where you called
// `misc_register`.
unsafe impl<T> Send for MiscDeviceRegistration<T> {}
// SAFETY: All `&self` methods on this type are written to ensure that it is safe to call them in
// parallel.
unsafe impl<T> Sync for MiscDeviceRegistration<T> {}

impl<T: MiscDevice> MiscDeviceRegistration<T> {
    /// Register a misc device.
    pub fn register(opts: MiscDeviceOptions) -> impl PinInit<Self, Error> {
        try_pin_init!(Self {
            inner <- Opaque::try_ffi_init(move |slot: *mut bindings::miscdevice| {
                // SAFETY: The initializer can write to the provided `slot`.
                unsafe { slot.write(opts.into_raw::<T>()) };

                // SAFETY: We just wrote the misc device options to the slot. The miscdevice will
                // get unregistered before `slot` is deallocated because the memory is pinned and
                // the destructor of this type deallocates the memory.
                // INVARIANT: If this returns `Ok(())`, then the `slot` will contain a registered
                // misc device.
                to_result(unsafe { bindings::misc_register(slot) })
            }),
            _t: PhantomData,
        })
    }

    /// Returns a raw pointer to the misc device.
    pub fn as_raw(&self) -> *mut bindings::miscdevice {
        self.inner.get()
    }

    /// Access the `this_device` field.
    pub fn device(&self) -> &Device {
        // SAFETY: This can only be called after a successful register(), which always
        // initialises `this_device` with a valid device. Furthermore, the signature of this
        // function tells the borrow-checker that the `&Device` reference must not outlive the
        // `&MiscDeviceRegistration<T>` used to obtain it, so the last use of the reference must be
        // before the underlying `struct miscdevice` is destroyed.
        unsafe { Device::as_ref((*self.as_raw()).this_device) }
    }
}

#[pinned_drop]
impl<T> PinnedDrop for MiscDeviceRegistration<T> {
    fn drop(self: Pin<&mut Self>) {
        // SAFETY: We know that the device is registered by the type invariants.
        unsafe { bindings::misc_deregister(self.inner.get()) };
    }
}

/// Trait implemented by the private data of an open misc device.
#[vtable]
pub trait MiscDevice: Sized {
    /// What kind of pointer should `Self` be wrapped in.
    type Ptr: ForeignOwnable + Send + Sync;

    /// Called when the misc device is opened.
    ///
    /// The returned pointer will be stored as the private data for the file.
    fn open(_file: &File, _misc: &MiscDeviceRegistration<Self>) -> Result<Self::Ptr>;

    /// Called when the misc device is released.
    fn release(device: Self::Ptr, _file: &File) {
        drop(device);
    }

    /// Read from this miscdevice.
    fn read_iter(
        _device: <Self::Ptr as ForeignOwnable>::Borrowed<'_>,
        _kiocb: Kiocb<'_, Self::Ptr>,
        _iov: &mut IovIter,
    ) -> Result<usize> {
        kernel::build_error(VTABLE_DEFAULT_ERROR)
    }

    /// Write to this miscdevice.
    fn write_iter(
        _device: <Self::Ptr as ForeignOwnable>::Borrowed<'_>,
        _kiocb: Kiocb<'_, Self::Ptr>,
        _iov: &mut IovIter,
    ) -> Result<usize> {
        kernel::build_error(VTABLE_DEFAULT_ERROR)
    }

    /// Handler for ioctls.
    ///
    /// The `cmd` argument is usually manipulated using the utilties in [`kernel::ioctl`].
    ///
    /// [`kernel::ioctl`]: mod@crate::ioctl
    fn ioctl(
        _device: <Self::Ptr as ForeignOwnable>::Borrowed<'_>,
        _file: &File,
        _cmd: u32,
        _arg: usize,
    ) -> Result<isize> {
        kernel::build_error(VTABLE_DEFAULT_ERROR)
    }

    /// Handler for ioctls.
    ///
    /// Used for 32-bit userspace on 64-bit platforms.
    ///
    /// This method is optional and only needs to be provided if the ioctl relies on structures
    /// that have different layout on 32-bit and 64-bit userspace. If no implementation is
    /// provided, then `compat_ptr_ioctl` will be used instead.
    #[cfg(CONFIG_COMPAT)]
    fn compat_ioctl(
        _device: <Self::Ptr as ForeignOwnable>::Borrowed<'_>,
        _file: &File,
        _cmd: u32,
        _arg: usize,
    ) -> Result<isize> {
        kernel::build_error(VTABLE_DEFAULT_ERROR)
    }

    /// Show info for this fd.
    fn show_fdinfo(
        _device: <Self::Ptr as ForeignOwnable>::Borrowed<'_>,
        _m: &SeqFile,
        _file: &File,
    ) {
        kernel::build_error(VTABLE_DEFAULT_ERROR)
    }
}

/// Wrapper for the kernel's `struct kiocb`.
///
/// The type `T` represents the private data of the file.
pub struct Kiocb<'a, T> {
    inner: NonNull<bindings::kiocb>,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: ForeignOwnable> Kiocb<'a, T> {
    /// Get the private data in this kiocb.
    pub fn private_data(&self) -> <T as ForeignOwnable>::Borrowed<'a> {
        // SAFETY: The `kiocb` lets us access the private data.
        let private = unsafe { (*(*self.inner.as_ptr()).ki_filp).private_data };
        // SAFETY: The kiocb has shared access to the private data.
        unsafe { <T as ForeignOwnable>::borrow(private) }
    }

    /// Gets the current value of `ki_pos`.
    pub fn ki_pos(&self) -> loff_t {
        // SAFETY: The `kiocb` can access `ki_pos`.
        unsafe { (*self.inner.as_ptr()).ki_pos }
    }

    /// Gets a mutable reference to the `ki_pos` field.
    pub fn ki_pos_mut(&mut self) -> &mut loff_t {
        // SAFETY: The `kiocb` can access `ki_pos`.
        unsafe { &mut (*self.inner.as_ptr()).ki_pos }
    }
}

/// Wrapper for the kernel's `struct iov_iter`.
pub struct IovIter {
    inner: Opaque<bindings::iov_iter>,
}

impl IovIter {
    /// Gets a raw pointer to the contents.
    pub fn as_raw(&self) -> *mut bindings::iov_iter {
        self.inner.get()
    }
}

const fn create_vtable<T: MiscDevice>() -> &'static bindings::file_operations {
    const fn maybe_fn<T: Copy>(check: bool, func: T) -> Option<T> {
        if check {
            Some(func)
        } else {
            None
        }
    }

    struct VtableHelper<T: MiscDevice> {
        _t: PhantomData<T>,
    }
    impl<T: MiscDevice> VtableHelper<T> {
        const VTABLE: bindings::file_operations = bindings::file_operations {
            open: Some(fops_open::<T>),
            release: Some(fops_release::<T>),
            read_iter: maybe_fn(T::HAS_READ_ITER, fops_read_iter::<T>),
            write_iter: maybe_fn(T::HAS_WRITE_ITER, fops_write_iter::<T>),
            unlocked_ioctl: maybe_fn(T::HAS_IOCTL, fops_ioctl::<T>),
            #[cfg(CONFIG_COMPAT)]
            compat_ioctl: if T::HAS_COMPAT_IOCTL {
                Some(fops_compat_ioctl::<T>)
            } else if T::HAS_IOCTL {
                Some(bindings::compat_ptr_ioctl)
            } else {
                None
            },
            show_fdinfo: maybe_fn(T::HAS_SHOW_FDINFO, fops_show_fdinfo::<T>),
            // SAFETY: All zeros is a valid value for `bindings::file_operations`.
            ..unsafe { MaybeUninit::zeroed().assume_init() }
        };
    }

    &VtableHelper::<T>::VTABLE
}

/// # Safety
///
/// `file` and `inode` must be the file and inode for a file that is undergoing initialization.
/// The file must be associated with a `MiscDeviceRegistration<T>`.
unsafe extern "C" fn fops_open<T: MiscDevice>(
    inode: *mut bindings::inode,
    raw_file: *mut bindings::file,
) -> c_int {
    // SAFETY: The pointers are valid and for a file being opened.
    let ret = unsafe { bindings::generic_file_open(inode, raw_file) };
    if ret != 0 {
        return ret;
    }

    // SAFETY: The open call of a file can access the private data.
    let misc_ptr = unsafe { (*raw_file).private_data };

    // SAFETY: This is a miscdevice, so `misc_open()` set the private data to a pointer to the
    // associated `struct miscdevice` before calling into this method. Furthermore, `misc_open()`
    // ensures that the miscdevice can't be unregistered and freed during this call to `fops_open`.
    let misc = unsafe { &*misc_ptr.cast::<MiscDeviceRegistration<T>>() };

    // SAFETY:
    // * This underlying file is valid for (much longer than) the duration of `T::open`.
    // * There is no active fdget_pos region on the file on this thread.
    let file = unsafe { File::from_raw_file(raw_file) };

    let ptr = match T::open(file, misc) {
        Ok(ptr) => ptr,
        Err(err) => return err.to_errno(),
    };

    // This overwrites the private data with the value specified by the user, changing the type of
    // this file's private data. All future accesses to the private data is performed by other
    // fops_* methods in this file, which all correctly cast the private data to the new type.
    //
    // SAFETY: The open call of a file can access the private data.
    unsafe { (*raw_file).private_data = ptr.into_foreign().cast_mut() };

    0
}

/// # Safety
///
/// `file` and `inode` must be the file and inode for a file that is being released. The file must
/// be associated with a `MiscDeviceRegistration<T>`.
unsafe extern "C" fn fops_release<T: MiscDevice>(
    _inode: *mut bindings::inode,
    file: *mut bindings::file,
) -> c_int {
    // SAFETY: The release call of a file owns the private data.
    let private = unsafe { (*file).private_data };
    // SAFETY: The release call of a file owns the private data.
    let ptr = unsafe { <T::Ptr as ForeignOwnable>::from_foreign(private) };

    // SAFETY:
    // * The file is valid for the duration of this call.
    // * There is no active fdget_pos region on the file on this thread.
    T::release(ptr, unsafe { File::from_raw_file(file) });

    0
}

/// # Safety
///
/// `kiocb` and `iter` must be related to the file that is associated with a `MiscDeviceRegistration<T>`. <----- CHECK ME -------
unsafe extern "C" fn fops_read_iter<T: MiscDevice>(
    kiocb: *mut bindings::kiocb,
    iter: *mut bindings::iov_iter,
) -> isize {
    let kiocb = Kiocb {
        // SAFETY: The `kiocb` related to a give file is always valid ------------------------------------------ CHECK ME -------
        inner: unsafe { NonNull::new_unchecked(kiocb) },
        _phantom: PhantomData,
    };
    // SAFETY: Be still compiler <------------------------------------------------------------------------------ CHECK ME -------
    let iov = unsafe { &mut *iter.cast::<IovIter>() };
    // SAFETY: Be still compiler <------------------------------------------------------------------------------ CHECK ME -------
    let file = unsafe { (*kiocb.inner.as_ptr()).ki_filp };
    // SAFETY: The ioctl call of a file can access the private data.
    let private = unsafe { (*file).private_data };
    // SAFETY: read_iter calls can borrow the private data of the file.
    let device = unsafe { <T::Ptr as ForeignOwnable>::borrow(private) };

    match T::read_iter(device, kiocb, iov) {
        Ok(res) => res as isize,
        Err(err) => err.to_errno() as isize,
    }
}

/// # Safety
///
/// `kiocb` and `iter` must be related to the file that is associated with a `MiscDeviceRegistration<T>`. <----- CHECK ME -------
unsafe extern "C" fn fops_write_iter<T: MiscDevice>(
    kiocb: *mut bindings::kiocb,
    iter: *mut bindings::iov_iter,
) -> isize {
    let kiocb = Kiocb {
        // SAFETY: The `kiocb` related to a give file is always valid ------------------------------------------ CHECK ME -------
        inner: unsafe { NonNull::new_unchecked(kiocb) },
        _phantom: PhantomData,
    };
    // SAFETY: Be still compiler <------------------------------------------------------------------------------ CHECK ME -------
    let iov = unsafe { &mut *iter.cast::<IovIter>() };
    // SAFETY: Be still compiler <------------------------------------------------------------------------------ CHECK ME -------
    let file = unsafe { (*kiocb.inner.as_ptr()).ki_filp };
    // SAFETY: The ioctl call of a file can access the private data.
    let private = unsafe { (*file).private_data };
    // SAFETY: write_iter calls can borrow the private data of the file.
    let device = unsafe { <T::Ptr as ForeignOwnable>::borrow(private) };

    match T::write_iter(device, kiocb, iov) {
        Ok(res) => res as isize,
        Err(err) => err.to_errno() as isize,
    }
}

/// # Safety
///
/// `file` must be a valid file that is associated with a `MiscDeviceRegistration<T>`.
unsafe extern "C" fn fops_ioctl<T: MiscDevice>(
    file: *mut bindings::file,
    cmd: c_uint,
    arg: c_ulong,
) -> c_long {
    // SAFETY: The ioctl call of a file can access the private data.
    let private = unsafe { (*file).private_data };
    // SAFETY: Ioctl calls can borrow the private data of the file.
    let device = unsafe { <T::Ptr as ForeignOwnable>::borrow(private) };

    // SAFETY:
    // * The file is valid for the duration of this call.
    // * There is no active fdget_pos region on the file on this thread.
    let file = unsafe { File::from_raw_file(file) };

    match T::ioctl(device, file, cmd, arg) {
        Ok(ret) => ret as c_long,
        Err(err) => err.to_errno() as c_long,
    }
}

/// # Safety
///
/// `file` must be a valid file that is associated with a `MiscDeviceRegistration<T>`.
#[cfg(CONFIG_COMPAT)]
unsafe extern "C" fn fops_compat_ioctl<T: MiscDevice>(
    file: *mut bindings::file,
    cmd: c_uint,
    arg: c_ulong,
) -> c_long {
    // SAFETY: The compat ioctl call of a file can access the private data.
    let private = unsafe { (*file).private_data };
    // SAFETY: Ioctl calls can borrow the private data of the file.
    let device = unsafe { <T::Ptr as ForeignOwnable>::borrow(private) };

    // SAFETY:
    // * The file is valid for the duration of this call.
    // * There is no active fdget_pos region on the file on this thread.
    let file = unsafe { File::from_raw_file(file) };

    match T::compat_ioctl(device, file, cmd, arg) {
        Ok(ret) => ret as c_long,
        Err(err) => err.to_errno() as c_long,
    }
}

/// # Safety
///
/// - `file` must be a valid file that is associated with a `MiscDeviceRegistration<T>`.
/// - `seq_file` must be a valid `struct seq_file` that we can write to.
unsafe extern "C" fn fops_show_fdinfo<T: MiscDevice>(
    seq_file: *mut bindings::seq_file,
    file: *mut bindings::file,
) {
    // SAFETY: The release call of a file owns the private data.
    let private = unsafe { (*file).private_data };
    // SAFETY: Ioctl calls can borrow the private data of the file.
    let device = unsafe { <T::Ptr as ForeignOwnable>::borrow(private) };
    // SAFETY:
    // * The file is valid for the duration of this call.
    // * There is no active fdget_pos region on the file on this thread.
    let file = unsafe { File::from_raw_file(file) };
    // SAFETY: The caller ensures that the pointer is valid and exclusive for the duration in which
    // this method is called.
    let m = unsafe { SeqFile::from_raw(seq_file) };

    T::show_fdinfo(device, m, file);
}
