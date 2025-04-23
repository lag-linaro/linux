// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2025 Google LLC
 *
 * Author: Lee Jones <lee@kernel.org>
 */

#include <kunit/test.h>

struct led_test_ddata {
	struct led_classdev cdev;
	struct device *dev;              /* TODO: What's the difference between this and cdev->dev after registration */
}

static void led_test_class_register(struct kunit *test)
{
	struct led_classdev *cdev = test->priv->cdev;
	struct device *dev = test->priv->dev;

	ret = devm_led_classdev_register(dev, cdev);
	KUNIT_ASSERT_EQ(test, ret, 0);
	if (ret)
		return;
}

static struct kunit_case led_test_cases[] = {
	KUNIT_CASE(led_test_class_register)
};

static int led_test_init(struct kunit *test)
{
	struct led_test_ddata *ddata;
	struct device *dev;

	ddata = kunit_kzalloc(test, sizeof(*ddata), GFP_KERNEL);
	if (!ddata)
		return -ENOMEM;

	test->priv = ddata;

	dev = kunit_device_register(test, "led_regmap");
	if (IS_ERR(dev)
		return PTR_ERR(dev);

	ddata->dev = get_device(dev);

	return 0;
}

static void led_test_exit(struct kunit *test)
{
	struct leds_test_ddata *ddata = test->priv;

	if (priv && priv->dev)
		put_device(priv->dev);
}

static struct kunit_suite led_test_suite = {
	.name = "led",
	.init = led_test_init,
	.exit = led_test_exit,
	.test_cases = led_test_cases,
};
kunit_test_suite(led_test_suite);

MODULE_DESCRIPTION("LED KUnit tests");
MODULE_LICENSE("GPL");
