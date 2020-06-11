// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) Linaro Ltd. 2020
 *
 * Author: Lee Jones <lee.jones@linaro.org>
 */

#include <linux/kernel.h>
#include <linux/module.h>
#include <linux/of.h>
#include <linux/of_device.h>
#include <linux/platform_device.h>

static int mfd_of_test_child_probe(struct platform_device *pdev)
{
	struct device_node *np = pdev->dev.of_node;
	struct device *dev = &pdev->dev;

	dev_info(dev, "Probing platform device: %d\n", pdev->id);

	if (np)
		dev_info(dev, "Using OF node: %s\n", np->full_name);
	else
		dev_info(dev, "No OF node associated with this device");

	return 0;
}

static const struct of_device_id mfd_of_test_child_match[] = {
       { .compatible = "mfd,of-test-child" },
       { },
};
MODULE_DEVICE_TABLE(of, mfd_of_test_child_match);

static struct platform_driver mfd_of_test_child_driver = {
	.driver = {
		.name = "mfd-of-test-child",
		.of_match_table = mfd_of_test_child_match,
	},
	.probe	= mfd_of_test_child_probe,
};
module_platform_driver(mfd_of_test_child_driver);

MODULE_AUTHOR("Lee Jones <lee.jones@linaro.org>");
MODULE_DESCRIPTION("MFD OF testing child driver");
MODULE_LICENSE("GPL v2");

