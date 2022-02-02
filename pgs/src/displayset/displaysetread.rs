/*
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a
 * copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 William Swartzendruber
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use super::{
    Cid,
    Composition,
    CompositionObject,
    DisplaySet,
    Object,
    Palette,
    PaletteEntry,
    Vid,
    Window,
    super::segment::{
        ReadError as SegmentReadError,
        ReadSegmentExt,
        Segment,
    },
};
use std::{
    collections::BTreeMap,
    io::Read,
};
use thiserror::Error as ThisError;

/// A specialized [`Result`](std::result::Result) type for display set-reading operations.
pub type ReadResult<T> = Result<T, ReadError>;

/// The error type for [ReadDisplaySetExt].
///
/// Errors are caused by either an invalid combination of segments, an invalid bitstream, or by
/// an underlying I/O error.
#[derive(ThisError, Debug)]
pub enum ReadError {
    /// The display set could not be read because of an underlying segment error.
    #[error("segment value error")]
    SegmentError {
        #[from]
        source: SegmentReadError,
    },
    /// The first segment in the display set was not a presentation composition segment (PCS).
    #[error("first segment is not a presentation composition segment")]
    MissingPresentationCompositionSegment,
    /// A segment has been encountered after an end segment (ES) was processed.
    #[error("segment encountered after end segment")]
    SegmentAfterEnd,
    /// The segments within the display set do not have consistent PTS values.
    #[error("PTS is not consistent with presentation composition segment")]
    InconsistentPts,
    /// The segments within the display set do not have consistent DTS values.
    #[error("DTS is not consistent with presentation composition segment")]
    InconsistentDts,
    /// A presentation composition segment (PCS) has been encountered outside of the first
    /// segment.
    #[error("unexpected presentation composition segment within display set")]
    UnexpectedPresentationCompositionSegment,
    /// A single window ID has been defined twice.
    #[error("duplicate window ID detected")]
    DuplicateWindowId,
    /// A single palette ID of the same version has been defined twice.
    #[error("duplicate palette ID and version detected")]
    DuplicatePaletteVid,
    /// A single object ID of the same version has been defined twice.
    #[error("duplicate object ID and version detected")]
    DuplicateObjectVid,
    /// The display set composition definition references an unknown object ID.
    #[error("composition references unknown object ID")]
    CompositionReferencesUnknownObjectId,
    /// The display set composition definition references an unknown window ID.
    #[error("composition references unknown window ID")]
    CompositionReferencesUnknownWindowId,
    /// A palette update sequence references an unknown palette ID.
    #[error("palette update references unknown palette ID")]
    PaletteUpdateReferencesUnknownPaletteId,
    /// The sequence state of an object definition segment (ODS) is invalid.
    #[error("invalid object sequence state")]
    InvalidObjectSequence,
    /// The display set contains an incomplete multi-part object.
    #[error("incomplete object sequence")]
    IncompleteObjectSequence,
    /// The different portions of a compound object have inconsistent IDs.
    #[error("object portions have inconsistent IDs")]
    InconsistentObjectId,
    /// The different portions of a compound object have inconsistent versions.
    #[error("object portions have inconsistent versions")]
    InconsistentObjectVersion,
    /// The bitstream declares an incomplete RLE sequence within an object definition segment
    /// (ODS).
    #[error("incomplete RLE sequence")]
    IncompleteRleSequence,
    /// The bitstream declares an invalid RLE sequence within an object definition segment
    /// (ODS).
    #[error("invalid RLE sequence")]
    InvalidRleSequence,
    /// The bitstream declares an incomplete RLE line within an object definition segment (ODS).
    #[error("incomplete RLE line")]
    IncompleteRleLine,
}

#[derive(PartialEq)]
enum Sequence {
    Single,
    Initial,
    Middle,
    Final,
}

/// Allows reading display sets from a source.
pub trait ReadDisplaySetExt {
    /// Reads the next segment from a source.
    fn read_display_set(&mut self) -> ReadResult<DisplaySet>;
}

impl<T> ReadDisplaySetExt for T where
    T: Read,
{
    fn read_display_set(&mut self) -> ReadResult<DisplaySet> {

        let mut segments = Vec::<Segment>::new();

        match self.read_segment()? {
            Segment::PresentationComposition(pcs) => {
                segments.push(Segment::PresentationComposition(pcs));
            }
            _ => {
                return Err(ReadError::MissingPresentationCompositionSegment)
            }
        };

        loop {
            match self.read_segment()? {
                Segment::PresentationComposition(_) => {
                    return Err(ReadError::UnexpectedPresentationCompositionSegment)
                }
                Segment::End(es) => {
                    segments.push(Segment::End(es));
                    break
                }
                segment => {
                    segments.push(segment);
                }
            }
        }

        DisplaySet::try_from(&mut segments[..])
    }
}

impl TryFrom<&mut [Segment]> for DisplaySet {

    type Error = ReadError;

    fn try_from(value: &mut [Segment]) -> Result<Self, Self::Error> {

        let mut pcs = None;
        let mut es = None;
        let mut sequence = Sequence::Single;
        let mut initial_object = None;
        let mut middle_objects = Vec::new();
        let mut windows = BTreeMap::<u8, Window>::new();
        let mut palettes = BTreeMap::<Vid<u8>, Palette>::new();
        let mut objects = BTreeMap::<Vid<u16>, Object>::new();
        let mut composition_objects = BTreeMap::<Cid, CompositionObject>::new();

        for segment in value.iter_mut() {

            if es.is_some() {
                return Err(ReadError::SegmentAfterEnd)
            }

            match segment {
                Segment::PresentationComposition(this_pcs) => {
                    if pcs.is_none() {
                        pcs = Some(this_pcs)
                    } else {
                        return Err(ReadError::UnexpectedPresentationCompositionSegment)
                    }
                }
                Segment::WindowDefinition(wds) => {
                    match &pcs {
                        Some(the_pcs) => {
                            if wds.pts != the_pcs.pts {
                                return Err(ReadError::InconsistentPts)
                            }
                            if wds.dts != the_pcs.dts {
                                return Err(ReadError::InconsistentDts)
                            }
                            for wd in wds.windows.iter() {
                                if windows.contains_key(&wd.id) {
                                    return Err(ReadError::DuplicateWindowId)
                                }
                                windows.insert(
                                    wd.id,
                                    Window {
                                        x: wd.x,
                                        y: wd.y,
                                        width: wd.width,
                                        height: wd.height,
                                    },
                                );
                            }
                        }
                        None => {
                            return Err(ReadError::MissingPresentationCompositionSegment)
                        }
                    }
                }
                Segment::PaletteDefinition(pds) => {
                    match &pcs {
                        Some(the_pcs) => {
                            if pds.pts != the_pcs.pts {
                                return Err(ReadError::InconsistentPts)
                            }
                            if pds.dts != the_pcs.dts {
                                return Err(ReadError::InconsistentDts)
                            }
                            let vid = Vid {
                                id: pds.id,
                                version: pds.version,
                            };
                            if palettes.contains_key(&vid) {
                                return Err(ReadError::DuplicatePaletteVid)
                            }
                            palettes.insert(
                                vid,
                                Palette {
                                    entries: pds.entries.iter().map(|pe|
                                        (pe.id, PaletteEntry {
                                            y: pe.y,
                                            cr: pe.cr,
                                            cb: pe.cb,
                                            alpha: pe.alpha,
                                        })
                                    ).collect::<BTreeMap<u8, PaletteEntry>>()
                                },
                            );
                        }
                        None => {
                            return Err(ReadError::MissingPresentationCompositionSegment)
                        }
                    }
                }
                Segment::SingleObjectDefinition(sods) => {
                    match &pcs {
                        Some(the_pcs) => {
                            if sequence == Sequence::Single || sequence == Sequence::Final {
                                if sods.pts != the_pcs.pts {
                                    return Err(ReadError::InconsistentPts)
                                }
                                if sods.dts != the_pcs.dts {
                                    return Err(ReadError::InconsistentDts)
                                }
                                let vid = Vid {
                                    id: sods.id,
                                    version: sods.version,
                                };
                                if objects.contains_key(&vid) {
                                    return Err(ReadError::DuplicateObjectVid)
                                }
                                objects.insert(
                                    vid,
                                    Object {
                                        width: sods.width,
                                        height: sods.height,
                                        lines: rle_decompress(&sods.data)?,
                                    },
                                );
                                sequence = Sequence::Single;
                            } else {
                                return Err(ReadError::InvalidObjectSequence)
                            }
                        }
                        None => {
                            return Err(ReadError::MissingPresentationCompositionSegment)
                        }
                    }
                }
                Segment::InitialObjectDefinition(iods) => {
                    match &pcs {
                        Some(the_pcs) => {
                            if sequence == Sequence::Single || sequence == Sequence::Final {
                                if iods.pts != the_pcs.pts {
                                    return Err(ReadError::InconsistentPts)
                                }
                                if iods.dts != the_pcs.dts {
                                    return Err(ReadError::InconsistentDts)
                                }
                                let vid = Vid {
                                    id: iods.id,
                                    version: iods.version,
                                };
                                if objects.contains_key(&vid) {
                                    return Err(ReadError::DuplicateObjectVid)
                                }
                                initial_object = Some(iods);
                                sequence = Sequence::Initial;
                            } else {
                                return Err(ReadError::InvalidObjectSequence)
                            }
                        }
                        None => {
                            return Err(ReadError::MissingPresentationCompositionSegment)
                        }
                    }
                }
                Segment::MiddleObjectDefinition(mods) => {
                    match &pcs {
                        Some(the_pcs) => {
                            if sequence == Sequence::Initial || sequence == Sequence::Middle {
                                match &initial_object {
                                    Some(iods) => {
                                        if mods.pts != the_pcs.pts {
                                            return Err(ReadError::InconsistentPts)
                                        }
                                        if mods.dts != the_pcs.dts {
                                            return Err(ReadError::InconsistentDts)
                                        }
                                        if mods.id != iods.id {
                                            return Err(ReadError::InconsistentObjectId)
                                        }
                                        if mods.version != iods.version {
                                            return Err(ReadError::InconsistentObjectVersion)
                                        }
                                        middle_objects.push(mods);
                                        sequence = Sequence::Middle;
                                    }
                                    None => {
                                        panic!("initial_object is not set")
                                    }
                                }
                            } else {
                                return Err(ReadError::InvalidObjectSequence)
                            }
                        }
                        None => {
                            return Err(ReadError::MissingPresentationCompositionSegment)
                        }
                    }
                }
                Segment::FinalObjectDefinition(fods) => {
                    match &pcs {
                        Some(the_pcs) => {
                            if sequence == Sequence::Initial || sequence == Sequence::Middle {
                                match &mut initial_object {
                                    Some(iods) => {
                                        if fods.pts != the_pcs.pts {
                                            return Err(ReadError::InconsistentPts)
                                        }
                                        if fods.dts != the_pcs.dts {
                                            return Err(ReadError::InconsistentDts)
                                        }
                                        if fods.id != iods.id {
                                            return Err(ReadError::InconsistentObjectId)
                                        }
                                        if fods.version != iods.version {
                                            return Err(ReadError::InconsistentObjectVersion)
                                        }
                                        let vid = Vid {
                                            id: iods.id,
                                            version: iods.version,
                                        };
                                        let mut data = Vec::new();
                                        data.append(&mut iods.data);
                                        for mods in middle_objects.iter_mut() {
                                            data.append(&mut mods.data);
                                        }
                                        data.append(&mut fods.data);
                                        objects.insert(
                                            vid,
                                            Object {
                                                width: iods.width,
                                                height: iods.height,
                                                lines: rle_decompress(&data)?,
                                            },
                                        );
                                        initial_object = None;
                                        middle_objects.clear();
                                        sequence = Sequence::Final;
                                    }
                                    None => {
                                        panic!("initial_object is not set")
                                    }
                                }
                            } else {
                                return Err(ReadError::InvalidObjectSequence)
                            }
                        }
                        None => {
                            return Err(ReadError::MissingPresentationCompositionSegment)
                        }
                    }
                }
                Segment::End(this_es) => {
                    match &pcs {
                        Some(the_pcs) => {
                            if sequence != Sequence::Single && sequence != Sequence::Final {
                                return Err(ReadError::IncompleteObjectSequence)
                            }
                            if this_es.pts != the_pcs.pts {
                                return Err(ReadError::InconsistentPts)
                            }
                            if this_es.dts != the_pcs.dts {
                                return Err(ReadError::InconsistentDts)
                            }
                            es = Some(this_es);
                        }
                        None => {
                            return Err(ReadError::MissingPresentationCompositionSegment)
                        }
                    }
                }
            }
        }

        let the_pcs = pcs.expect("PCS is not set, somehow");

        for co in the_pcs.composition_objects.iter() {
            // TODO: Maybe re-enable.
            // if !objects.keys().any(|vid| vid.id == co.object_id) {
            //     return Err(ReadError::CompositionReferencesUnknownObjectId)
            // }
            // if !windows.contains_key(&co.window_id) {
            //     return Err(ReadError::CompositionReferencesUnknownWindowId)
            // }
            composition_objects.insert(
                Cid {
                    object_id: co.object_id,
                    window_id: co.window_id,
                },
                CompositionObject {
                    x: co.x,
                    y: co.y,
                    crop: co.crop.clone(),
                },
            );
        }

        let composition = Composition {
            number: the_pcs.composition_number,
            state: the_pcs.composition_state,
            objects: composition_objects,
        };

        match the_pcs.palette_update_id {
            Some(palette_update_id) => {
                if !palettes.keys().any(|vid| vid.id == palette_update_id) {
                    return Err(ReadError::PaletteUpdateReferencesUnknownPaletteId)
                }
            }
            None => {
            }
        }

        Ok(
            DisplaySet {
                pts: the_pcs.pts,
                dts: the_pcs.dts,
                width: the_pcs.width,
                height: the_pcs.height,
                frame_rate: the_pcs.frame_rate,
                palette_update_id: the_pcs.palette_update_id,
                windows,
                palettes,
                objects,
                composition,
            }
        )
    }
}

fn rle_decompress(input: &[u8]) -> ReadResult<Vec<Vec<u8>>> {

    let mut output = Vec::<Vec<u8>>::new();
    let mut line = vec![];
    let mut iter = input.iter();

    loop {
        match iter.next() {
            Some(byte_1) => {
                if *byte_1 == 0x00 {
                    match iter.next() {
                        Some(byte_2) => {
                            if *byte_2 == 0x00 {
                                output.push(line);
                                line = vec![];
                            } else if *byte_2 >> 6 == 0 {
                                for _ in 0..(*byte_2 & 0x3F) {
                                    line.push(0);
                                }
                            } else if *byte_2 >> 6 == 1 {
                                match iter.next() {
                                    Some(byte_3) => {
                                        for _ in 0..(
                                            (*byte_2 as u16 & 0x3F) << 8
                                            | *byte_3 as u16
                                        ) {
                                            line.push(0);
                                        }
                                    }
                                    None => {
                                        return Err(ReadError::IncompleteRleSequence)
                                    }
                                }
                            } else if *byte_2 >> 6 == 2 {
                                match iter.next() {
                                    Some(byte_3) => {
                                        for _ in 0..(*byte_2 & 0x3F) {
                                            line.push(*byte_3);
                                        }
                                    }
                                    None => {
                                        return Err(ReadError::IncompleteRleSequence)
                                    }
                                }
                            } else if *byte_2 >> 6 == 3 {
                                match iter.next() {
                                    Some(byte_3) => {
                                        match iter.next() {
                                            Some(byte_4) => {
                                                for _ in 0..(
                                                    (*byte_2 as u16 & 0x3F) << 8
                                                    | *byte_3 as u16
                                                ) {
                                                    line.push(*byte_4);
                                                }
                                            }
                                            None => {
                                                return Err(ReadError::IncompleteRleSequence)
                                            }
                                        }
                                    }
                                    None => {
                                        return Err(ReadError::IncompleteRleSequence)
                                    }
                                }
                            } else {
                                return Err(ReadError::InvalidRleSequence)
                            }
                        }
                        None => {
                            return Err(ReadError::IncompleteRleSequence)
                        }
                    }
                } else {
                    line.push(*byte_1);
                }
            }
            None => {
                break
            }
        }
    }

    if !line.is_empty() {
        return Err(ReadError::IncompleteRleLine)
    }

    Ok(output)
}
