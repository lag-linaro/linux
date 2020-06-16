// SPDX-License-Identifier: GPL-2.0-or-later
/*
 * System Control Driver accessed over I2C
 *
 * Copyright (C) 2020 Linaro Ltd.
 *
 * Author: Lee Jones <lee.jones@linaro.org>
 */

#include <linux/err.h>
#include <linux/i2c.h>
#include <linux/list.h>
#include <linux/mfd/syscon-i2c.h>
#include <linux/regmap.h>

static DEFINE_SPINLOCK(syscon_i2c_list_slock);
static LIST_HEAD(syscon_i2c_list);

struct syscon {
	struct device_node *np;
	struct regmap *regmap;
	struct list_head list;
};

static const struct regmap_config syscon_i2c_regmap_config = {
	.reg_bits = 8,
	.val_bits = 8,
};

static struct syscon *of_syscon_i2c_register(struct i2c_client *client)
{
	struct regmap_config syscon_config = syscon_i2c_regmap_config;
	struct device_node *np = client->dev.of_node;
	struct syscon *syscon;
	struct regmap *regmap;
	int ret;

	syscon = devm_kzalloc(&client->dev, sizeof(*syscon), GFP_KERNEL);
	if (!syscon)
		return ERR_PTR(-ENOMEM);

	syscon_config.name = of_node_full_name(np);

	regmap = devm_regmap_init_i2c(client, &syscon_config);
	if (IS_ERR(regmap)) {
		dev_err(&client->dev, "Failed to initialise Regmap I2C\n");
		ret = PTR_ERR(regmap);
		return ERR_PTR(ret);
	}

	syscon->regmap = regmap;
	syscon->np = np;

	spin_lock(&syscon_i2c_list_slock);
	list_add_tail(&syscon->list, &syscon_i2c_list);
	spin_unlock(&syscon_i2c_list_slock);

	return syscon;
}

static struct regmap *i2c_device_node_get_regmap(struct i2c_client *client)
{
	struct device_node *np = client->dev.of_node;
	struct syscon *entry, *syscon = NULL;

	spin_lock(&syscon_i2c_list_slock);

	list_for_each_entry(entry, &syscon_i2c_list, list)
		if (entry->np == np) {
			syscon = entry;
			break;
		}

	spin_unlock(&syscon_i2c_list_slock);

	if (!syscon)
		syscon = of_syscon_i2c_register(client);

	if (IS_ERR(syscon))
		return ERR_CAST(syscon);

	return syscon->regmap;
}

struct regmap *i2c_device_node_to_regmap(struct i2c_client *client)
{
	return i2c_device_node_get_regmap(client);
}
EXPORT_SYMBOL_GPL(i2c_device_node_to_regmap);
