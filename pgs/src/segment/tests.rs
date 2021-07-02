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
    let segment = Segment::PresentationComposition(
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

    cycle(&segment);
}

#[test]
fn test_pcs_cycle_no_pui_co() {

    let mut rng = thread_rng();
    let segment = Segment::PresentationComposition(
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

    cycle(&segment);
}

#[test]
fn test_pcs_cycle_pui_no_co() {

    let mut rng = thread_rng();
    let segment = Segment::PresentationComposition(
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

    cycle(&segment);
}

#[test]
fn test_pcs_cycle_pui_co() {

    let mut rng = thread_rng();
    let segment = Segment::PresentationComposition(
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

    cycle(&segment);
}

#[test]
fn test_wds_empty() {

    let mut rng = thread_rng();
    let segment = Segment::WindowDefinition(
        WindowDefinitionSegment {
            pts: rng.gen(),
            dts: rng.gen(),
            windows: vec![],
        }
    );

    cycle(&segment);
}

#[test]
fn test_wds_not_empty() {

    let mut rng = thread_rng();
    let segment = Segment::WindowDefinition(
        WindowDefinitionSegment {
            pts: rng.gen(),
            dts: rng.gen(),
            windows: vec![
                WindowDefinition {
                    id: rng.gen(),
                    x: rng.gen(),
                    y: rng.gen(),
                    width: rng.gen(),
                    height: rng.gen(),
                },
                WindowDefinition {
                    id: rng.gen(),
                    x: rng.gen(),
                    y: rng.gen(),
                    width: rng.gen(),
                    height: rng.gen(),
                },
                WindowDefinition {
                    id: rng.gen(),
                    x: rng.gen(),
                    y: rng.gen(),
                    width: rng.gen(),
                    height: rng.gen(),
                },
            ],
        }
    );

    cycle(&segment);
}

#[test]
fn test_pds_empty() {

    let mut rng = thread_rng();
    let segment = Segment::PaletteDefinition(
        PaletteDefinitionSegment {
            pts: rng.gen(),
            dts: rng.gen(),
            id: rng.gen(),
            version: rng.gen(),
            entries: vec![],
        }
    );

    cycle(&segment);
}

#[test]
fn test_pds_not_empty() {

    let mut rng = thread_rng();
    let segment = Segment::PaletteDefinition(
        PaletteDefinitionSegment {
            pts: rng.gen(),
            dts: rng.gen(),
            id: rng.gen(),
            version: rng.gen(),
            entries: vec![
                PaletteEntry {
                    id: rng.gen(),
                    y: rng.gen(),
                    cr: rng.gen(),
                    cb: rng.gen(),
                    alpha: rng.gen(),
                },
                PaletteEntry {
                    id: rng.gen(),
                    y: rng.gen(),
                    cr: rng.gen(),
                    cb: rng.gen(),
                    alpha: rng.gen(),
                },
                PaletteEntry {
                    id: rng.gen(),
                    y: rng.gen(),
                    cr: rng.gen(),
                    cb: rng.gen(),
                    alpha: rng.gen(),
                },
            ],
        }
    );

    cycle(&segment);
}

#[test]
fn test_ods_single() {

    let mut rng = thread_rng();
    let segment = Segment::ObjectDefinition(
        ObjectDefinitionSegment {
            pts: rng.gen(),
            dts: rng.gen(),
            id: rng.gen(),
            version: rng.gen(),
            sequence: Sequence::Single,
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
        }
    );

    cycle(&segment);
}

#[test]
fn test_ods_first() {

    let mut rng = thread_rng();
    let segment = Segment::ObjectDefinition(
        ObjectDefinitionSegment {
            pts: rng.gen(),
            dts: rng.gen(),
            id: rng.gen(),
            version: rng.gen(),
            sequence: Sequence::First,
            width: rng.gen(),
            height: rng.gen(),
            lines: vec![],
        }
    );

    cycle(&segment);
}

#[test]
fn test_ods_last() {

    let mut rng = thread_rng();
    let segment = Segment::ObjectDefinition(
        ObjectDefinitionSegment {
            pts: rng.gen(),
            dts: rng.gen(),
            id: rng.gen(),
            version: rng.gen(),
            sequence: Sequence::Last,
            width: rng.gen(),
            height: rng.gen(),
            lines: vec![],
        }
    );

    cycle(&segment);
}

#[test]
fn test_es() {

    let mut rng = thread_rng();
    let segment = Segment::End(
        EndSegment {
            pts: rng.gen(),
            dts: rng.gen(),
        }
    );

    cycle(&segment);
}

fn cycle(segment: &Segment) {

    let mut buffer = vec![];

    buffer.write_segment(&segment).unwrap();

    let mut cursor = Cursor::new(buffer);
    let cycled_segment = cursor.read_segment().unwrap();

    assert_eq!(cycled_segment, *segment);
}
