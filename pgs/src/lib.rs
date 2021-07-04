/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: OSL-3.0
 */

pub mod displayset;
pub mod segment;

pub fn ts_to_timestamp(ts: u32) -> String {

    let mut ms = ts / 90;
    let h = ms / 3_600_000;
    ms -= h * 3_600_000;
    let m = ms / 60_000;
    ms -= m * 60_000;
    let s = ms / 1_000;
    ms -= s * 1_000;

    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms)
}
