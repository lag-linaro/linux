/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Copyright (C) 2020 Linaro Ltd.
 *
 * Author: Lee Jones <lee.jones@linaro.org>
 */

#ifndef _LINUX_PLATFORM_DATA_PASEMI_H
#define _LINUX_PLATFORM_DATA_PASEMI_H

/* Restore astate to last set */
#ifdef CONFIG_PPC_PASEMI_CPUFREQ
int check_astate(void);
void restore_astate(int cpu);
#else
static inline int check_astate(void)
{
        /* Always return >0 so we never power save */
        return 1;
}
static inline void restore_astate(int cpu)
{
}
#endif

#endif /* _LINUX_PLATFORM_DATA_PASEMI_H */


