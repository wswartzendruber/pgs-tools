/*
 * Any copyright is dedicated to the Public Domain.
 *
 * Copyright 2021 William Swartzendruber
 *
 * SPDX-License-Identifier: CC0-1.0
 */

use super::{
    *,
    super::segment::{CompositionState, Crop},
    displaysetread::ReadDisplaySetExt,
    displaysetwrite::WriteDisplaySetExt,
};
use std::{
    collections::BTreeMap,
    io::Cursor,
};
use rand::{thread_rng, Rng};

#[test]
fn test_ds_cycle_empty() {

    let mut rng = thread_rng();
    let mut buffer = vec![];

    let display_set = DisplaySet {
        pts: rng.gen(),
        dts: rng.gen(),
        width: rng.gen(),
        height: rng.gen(),
        frame_rate: rng.gen(),
        palette_update_id: None,
        windows: BTreeMap::<u8, Window>::new(),
        palettes: BTreeMap::<Vid<u8>, Palette>::new(),
        objects: BTreeMap::<Vid<u16>, Object>::new(),
        composition: Composition {
            number: rng.gen(),
            state: CompositionState::EpochStart,
            objects: BTreeMap::<Cid, CompositionObject>::new(),
        },
    };

    buffer.write_display_set(&display_set).unwrap();

    let mut cursor = Cursor::new(buffer);
    let cycled_display_set = cursor.read_display_set().unwrap();

    assert_eq!(cycled_display_set, display_set);
}

#[test]
fn test_ds_cycle_not_empty() {

    let mut rng = thread_rng();
    let mut buffer = vec![];
    let mut composition_objects = BTreeMap::<Cid, CompositionObject>::new();
    let mut windows = BTreeMap::<u8, Window>::new();
    let mut palettes = BTreeMap::<Vid<u8>, Palette>::new();
    let mut palette_entries = BTreeMap::<u8, PaletteEntry>::new();
    let mut objects = BTreeMap::<Vid<u16>, Object>::new();

    composition_objects.insert(
        Cid {
            object_id: 1,
            window_id: 1,
        },
        CompositionObject {
            x: rng.gen(),
            y: rng.gen(),
            crop: None,
        },
    );
    composition_objects.insert(
        Cid {
            object_id: 2,
            window_id: 2,
        },
        CompositionObject {
            x: rng.gen(),
            y: rng.gen(),
            crop: Some(Crop {
                x: rng.gen(),
                y: rng.gen(),
                width: rng.gen(),
                height: rng.gen(),
            }),
        },
    );
    composition_objects.insert(
        Cid {
            object_id: 3,
            window_id: 3,
        },
        CompositionObject {
            x: rng.gen(),
            y: rng.gen(),
            crop: Some(Crop {
                x: rng.gen(),
                y: rng.gen(),
                width: rng.gen(),
                height: rng.gen(),
            }),
        },
    );

    windows.insert(
        1,
        Window {
            x: rng.gen(),
            y: rng.gen(),
            width: rng.gen(),
            height: rng.gen(),
        },
    );
    windows.insert(
        2,
        Window {
            x: rng.gen(),
            y: rng.gen(),
            width: rng.gen(),
            height: rng.gen(),
        },
    );
    windows.insert(
        3,
        Window {
            x: rng.gen(),
            y: rng.gen(),
            width: rng.gen(),
            height: rng.gen(),
        },
    );

    palette_entries.insert(
        1,
        PaletteEntry {
            y: rng.gen(),
            cr: rng.gen(),
            cb: rng.gen(),
            alpha: rng.gen(),
        },
    );
    palette_entries.insert(
        2,
        PaletteEntry {
            y: rng.gen(),
            cr: rng.gen(),
            cb: rng.gen(),
            alpha: rng.gen(),
        },
    );
    palette_entries.insert(
        3,
        PaletteEntry {
            y: rng.gen(),
            cr: rng.gen(),
            cb: rng.gen(),
            alpha: rng.gen(),
        },
    );

    palettes.insert(
        Vid {
            id: 1,
            version: 1,
        },
        Palette {
            entries: palette_entries,
        },
    );

    objects.insert(
        Vid {
            id: 1,
            version: 1,
        },
        Object {
            width: rng.gen(),
            height: rng.gen(),
            lines: vec![
                vec![],
                vec![0],
                vec![],
                vec![1],
                vec![],
                vec![0, 0],
                vec![],
                vec![1, 1],
                vec![],
                vec![0, 0, 0],
                vec![],
                vec![1, 1, 1],
                vec![],
                vec![
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                vec![],
                vec![
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                ],
                vec![],
                vec![
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                vec![],
                vec![
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                ],
                vec![],
                vec![],
                vec![
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                vec![],
                vec![
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                ],
                vec![
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
                    21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
                    40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58,
                    59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77,
                    78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96,
                    97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112,
                    113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127,
                    128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142,
                    143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157,
                    158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172,
                    173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187,
                    188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202,
                    203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217,
                    218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232,
                    233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247,
                    248, 249, 250, 251, 252, 253, 254, 255,
                ],
                vec![],
                vec![],
            ],
        },
    );
    objects.insert(
        Vid {
            id: 2,
            version: 1,
        },
        Object {
            width: rng.gen(),
            height: rng.gen(),
            lines: vec![],
        },
    );
    objects.insert(
        Vid {
            id: 3,
            version: 1,
        },
        Object {
            width: rng.gen(),
            height: rng.gen(),
            lines: vec![],
        },
    );

    let display_set = DisplaySet {
        pts: rng.gen(),
        dts: rng.gen(),
        width: rng.gen(),
        height: rng.gen(),
        frame_rate: rng.gen(),
        palette_update_id: None,
        windows,
        palettes,
        objects,
        composition: Composition {
            number: rng.gen(),
            state: CompositionState::EpochStart,
            objects: composition_objects,
        },
    };

    buffer.write_display_set(&display_set).unwrap();

    let mut cursor = Cursor::new(buffer);
    let cycled_display_set = cursor.read_display_set().unwrap();

    assert_eq!(cycled_display_set, display_set);
}
