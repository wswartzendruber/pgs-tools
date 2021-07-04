/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
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
            sequence: Sequence::Single,
            lines: vec![],
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
            sequence: Sequence::Single,
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
            sequence: Sequence::Single,
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
