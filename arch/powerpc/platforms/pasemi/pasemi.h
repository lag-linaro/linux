/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _PASEMI_PASEMI_H
#define _PASEMI_PASEMI_H

extern time64_t pas_get_boot_time(void);
extern void pas_pci_init(void);
extern void pas_pci_irq_fixup(struct pci_dev *dev);
extern void pas_pci_dma_dev_setup(struct pci_dev *dev);

extern void __iomem *pasemi_pci_getcfgaddr(struct pci_dev *dev, int offset);

extern void __init pasemi_map_registers(void);

/* Power savings modes, implemented in asm */
extern void idle_spin(void);
extern void idle_doze(void);

extern struct pci_controller_ops pasemi_pci_controller_ops;

#endif /* _PASEMI_PASEMI_H */
