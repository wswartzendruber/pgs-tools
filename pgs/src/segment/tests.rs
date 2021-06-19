/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: CC0-1.0
 */

use super::{
    *,
    segmentread::ReadSegmentExt,
    segmentwrite::WriteSegmentExt,
};
use std::io::Cursor;
use rand::{thread_rng, Rng};

#[test]
fn test_pcs_cycle_no_pui_no_co() {

    let mut rng = thread_rng();
    let mut buffer = vec![];
    let out_pcs = Segment::PresentationComposition(
        PresentationCompositionSegment {
            pts: rng.gen(),
            dts: rng.gen(),
            width: rng.gen(),
            height: rng.gen(),
            frame_rate: rng.gen(),
            composition_number: rng.gen(),
            composition_state: CompositionState::Normal,
            palette_update_id: None,
            composition_objects: vec![],
        }
    );

    buffer.write_segment(&out_pcs).unwrap();

    let mut cursor = Cursor::new(buffer);
    let in_pcs = cursor.read_segment().unwrap();

    assert_eq!(out_pcs, in_pcs);
}

#[test]
fn test_pcs_cycle_no_pui_co() {

    let mut rng = thread_rng();
    let mut buffer = vec![];
    let out_pcs = Segment::PresentationComposition(
        PresentationCompositionSegment {
            pts: rng.gen(),
            dts: rng.gen(),
            width: rng.gen(),
            height: rng.gen(),
            frame_rate: rng.gen(),
            composition_number: rng.gen(),
            composition_state: CompositionState::Normal,
            palette_update_id: None,
            composition_objects: vec![
                CompositionObject {
                    object_id: rng.gen(),
                    window_id: rng.gen(),
                    x: rng.gen(),
                    y: rng.gen(),
                    crop: None,
                },
                CompositionObject {
                    object_id: rng.gen(),
                    window_id: rng.gen(),
                    x: rng.gen(),
                    y: rng.gen(),
                    crop: Some(
                        Crop {
                            x: rng.gen(),
                            y: rng.gen(),
                            width: rng.gen(),
                            height: rng.gen(),
                        }
                    ),
                },
            ],
        }
    );

    buffer.write_segment(&out_pcs).unwrap();

    let mut cursor = Cursor::new(buffer);
    let in_pcs = cursor.read_segment().unwrap();

    assert_eq!(out_pcs, in_pcs);
}

#[test]
fn test_pcs_cycle_pui_no_co() {

    let mut rng = thread_rng();
    let mut buffer = vec![];
    let out_pcs = Segment::PresentationComposition(
        PresentationCompositionSegment {
            pts: rng.gen(),
            dts: rng.gen(),
            width: rng.gen(),
            height: rng.gen(),
            frame_rate: rng.gen(),
            composition_number: rng.gen(),
            composition_state: CompositionState::Normal,
            palette_update_id: Some(rng.gen()),
            composition_objects: vec![],
        }
    );

    buffer.write_segment(&out_pcs).unwrap();

    let mut cursor = Cursor::new(buffer);
    let in_pcs = cursor.read_segment().unwrap();

    assert_eq!(out_pcs, in_pcs);
}

#[test]
fn test_pcs_cycle_pui_co() {

    let mut rng = thread_rng();
    let mut buffer = vec![];
    let out_pcs = Segment::PresentationComposition(
        PresentationCompositionSegment {
            pts: rng.gen(),
            dts: rng.gen(),
            width: rng.gen(),
            height: rng.gen(),
            frame_rate: rng.gen(),
            composition_number: rng.gen(),
            composition_state: CompositionState::Normal,
            palette_update_id: Some(rng.gen()),
            composition_objects: vec![
                CompositionObject {
                    object_id: rng.gen(),
                    window_id: rng.gen(),
                    x: rng.gen(),
                    y: rng.gen(),
                    crop: None,
                },
                CompositionObject {
                    object_id: rng.gen(),
                    window_id: rng.gen(),
                    x: rng.gen(),
                    y: rng.gen(),
                    crop: Some(
                        Crop {
                            x: rng.gen(),
                            y: rng.gen(),
                            width: rng.gen(),
                            height: rng.gen(),
                        }
                    ),
                },
            ],
        }
    );

    buffer.write_segment(&out_pcs).unwrap();

    let mut cursor = Cursor::new(buffer);
    let in_pcs = cursor.read_segment().unwrap();

    assert_eq!(out_pcs, in_pcs);
}
