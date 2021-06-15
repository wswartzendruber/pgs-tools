/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: CC0-1.0
 */

use super::*;

#[test]
fn test_every_possible_yuv_combination() {

    for y in 16..235 {
        for cb in 0..=255 {
            for cr in 0..=255 {

                let yuv = YcbcrGammaPixel { y, cb, cr };

                assert_eq!(yuv, ycbcr_gamma_pixel(rgb_linear_pixel(yuv)));
            }
        }
    }
}
