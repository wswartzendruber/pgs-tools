/*
 * Any copyright is dedicated to the Public Domain.
 * https://creativecommons.org/publicdomain/zero/1.0/
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
