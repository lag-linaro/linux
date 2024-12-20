// SPDX-License-Identifier: GPL-2.0

#include <linux/uio.h>

size_t rust_helper_iov_iter_count(const struct iov_iter *i)
{
	return iov_iter_count(i);
}

size_t rust_helper_copy_to_iter(const void *addr, size_t bytes, struct iov_iter *i)
{
	return copy_to_iter(addr, bytes, i);
}

size_t rust_helper_copy_from_iter(void *addr, size_t bytes, struct iov_iter *i)
{
	return copy_from_iter(addr, bytes, i);
}
