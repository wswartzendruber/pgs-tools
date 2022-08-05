/*
 * Copyright 2021 William Swartzendruber
 *
 * Any copyright is dedicated to the Public Domain.
 *
 * SPDX-License-Identifier: CC0-1.0
 */

use super::*;

#[test]
fn test_every_possible_yuv_combination() {

    for y in 16..235 {
        for cb in 0..=255 {
            for cr in 0..=255 {

                let yuv = YcbcrPixel { y, cb, cr };

                assert_eq!(yuv, ycbcr_pixel(rgb_pixel(yuv)));
            }
        }
    }
}
