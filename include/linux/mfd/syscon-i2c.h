/* SPDX-License-Identifier: GPL-2.0-or-later */
/*
 * System Control Driver accessed via I2C
 *
 * Copyright (C) 2020 Linaro Ltd.
 *
 * Author: Lee Jones <lee.jones@linaro.org>
 */

#ifndef __LINUX_MFD_SYSCON_I2C_H__
#define __LINUX_MFD_SYSCON_I2C_H__

#include <linux/err.h>
#include <linux/errno.h>
#include <linux/i2c.h>

#ifdef CONFIG_MFD_SYSCON_I2C
extern struct regmap *i2c_device_node_to_regmap(struct i2c_client *client);
#else
static inline struct regmap *i2c_device_node_to_regmap(struct i2c_client *client)
{
	return ERR_PTR(-ENOTSUPP);
}
#endif

#endif /* __LINUX_MFD_SYSCON_I2C_H__ */
