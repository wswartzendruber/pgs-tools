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
fn test_new_item_offset_simple() {
    assert_eq!(new_item_offset(800, 140, 88, 563, 40), 423);
}

#[test]
fn test_new_item_offset_too_high() {
    assert_eq!(new_item_offset(800, 140, 88, 95, 40), 40);
}

#[test]
fn test_new_item_offset_too_low() {
    assert_eq!(new_item_offset(800, 140, 88, 852, 40), 672);
}
