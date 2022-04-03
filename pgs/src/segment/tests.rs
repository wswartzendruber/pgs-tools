/*
 * Any copyright is dedicated to the Public Domain.
 *
 * Copyright 2022 William Swartzendruber
 *
 * SPDX-License-Identifier: CC0-1.0
 */

use super::{
    *,
    segmentread::ReadSegmentExt,
    segmentwrite::WriteSegmentExt,
};
use std::io::Cursor;
use rand::{thread_rng, Rng, RngCore};

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
                            flag: rng.gen(),
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
                            flag: rng.gen(),
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
    let mut data = Vec::with_capacity(1_024); rng.fill_bytes(&mut data);
    let segment = Segment::SingleObjectDefinition(
        SingleObjectDefinitionSegment {
            pts: rng.gen(),
            dts: rng.gen(),
            id: rng.gen(),
            version: rng.gen(),
            width: rng.gen(),
            height: rng.gen(),
            data,
        }
    );

    cycle(&segment);
}

#[test]
fn test_ods_first() {

    let mut rng = thread_rng();
    let mut data = Vec::with_capacity(1_024); rng.fill_bytes(&mut data);
    let segment = Segment::InitialObjectDefinition(
        InitialObjectDefinitionSegment {
            pts: rng.gen(),
            dts: rng.gen(),
            id: rng.gen(),
            version: rng.gen(),
            length: data.len(),
            width: rng.gen(),
            height: rng.gen(),
            data,
        }
    );

    cycle(&segment);
}

#[test]
fn test_ods_middle() {

    let mut rng = thread_rng();
    let mut data = Vec::with_capacity(1_024); rng.fill_bytes(&mut data);
    let segment = Segment::MiddleObjectDefinition(
        MiddleObjectDefinitionSegment {
            pts: rng.gen(),
            dts: rng.gen(),
            id: rng.gen(),
            version: rng.gen(),
            data,
        }
    );

    cycle(&segment);
}

#[test]
fn test_ods_last() {

    let mut rng = thread_rng();
    let mut data = Vec::with_capacity(1_024); rng.fill_bytes(&mut data);
    let segment = Segment::FinalObjectDefinition(
        FinalObjectDefinitionSegment {
            pts: rng.gen(),
            dts: rng.gen(),
            id: rng.gen(),
            version: rng.gen(),
            data,
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
