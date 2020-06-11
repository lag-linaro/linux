// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) Linaro Ltd. 2020
 *
 * Author: Lee Jones <lee.jones@linaro.org>
 */

#include <linux/kernel.h>
#include <linux/mfd/core.h>
#include <linux/module.h>
#include <linux/of.h>
#include <linux/of_device.h>
#include <linux/platform_device.h>

static const struct mfd_cell mfd_of_test_cell[] = {
//	OF_MFD_CELL("mfd-of-test-child", NULL, NULL, 0, 1, "mfd,of-test-child"),
	OF_MFD_CELL_REG("mfd-of-test-child", NULL, NULL, 0, 1, "mfd,of-test-child", 0xdddddddd00000000),
	OF_MFD_CELL_REG("mfd-of-test-child", NULL, NULL, 0, 2, "mfd,of-test-child", 0xaaaaaaaaaaaaaaaa),
	OF_MFD_CELL_REG("mfd-of-test-child", NULL, NULL, 0, 3, "mfd,of-test-child", 0x0000000000000000),
	OF_MFD_CELL_REG("mfd-of-test-child", NULL, NULL, 0, 4, "mfd,of-test-child", 0x00000000cccccccc)
};

static int mfd_of_test_probe(struct platform_device *pdev)
{
	struct device *dev = &pdev->dev;
	int ret;

	dev_info(dev, "Registering %ld devices", ARRAY_SIZE(mfd_of_test_cell));

	ret = devm_mfd_add_devices(dev, 0, mfd_of_test_cell,
				   ARRAY_SIZE(mfd_of_test_cell), NULL, 0, NULL);
	if (ret) {
		dev_err(dev, "Failed to register sub-devices\n");
		return ret;
	}

	return 0;
}

static const struct of_device_id mfd_of_test_match[] = {
       { .compatible = "mfd,of-test-parent" },
       { },
};
MODULE_DEVICE_TABLE(of, mfd_of_test_match);

static struct platform_driver mfd_of_test_driver = {
	.driver = {
		.name = "mfd-of-test-parent",
		.of_match_table = mfd_of_test_match,
	},
	.probe	= mfd_of_test_probe,
};
module_platform_driver(mfd_of_test_driver);

MODULE_AUTHOR("Lee Jones <lee.jones@linaro.org>");
MODULE_DESCRIPTION("MFD OF testing driver");
MODULE_LICENSE("GPL v2");

