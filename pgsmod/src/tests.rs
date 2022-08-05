/*
 * Copyright 2021 William Swartzendruber
 *
 * Any copyright is dedicated to the Public Domain.
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
