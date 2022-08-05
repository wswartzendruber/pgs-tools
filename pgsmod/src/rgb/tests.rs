/*
 * Copyright 2021 William Swartzendruber
 *
 * To the extent possible under law, the person who associated CC0 with this file has waived all
 * copyright and related or neighboring rights to this file.
 *
 * You should have received a copy of the CC0 legalcode along with this work. If not, see
 * <http://creativecommons.org/publicdomain/zero/1.0/>.
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
