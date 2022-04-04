/*
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a
 * copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2022 William Swartzendruber
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
    /// The display set contains no segments.
    #[error("no segments")]
    NoSegments,
    /// The first segment in the display set was not a presentation composition segment (PCS).
    #[error("first segment is not a presentation composition segment")]
    MissingPresentationCompositionSegment,
    /// A segment has been encountered after an end segment (ES) was processed.
    #[error("segment encountered after end segment")]
    SegmentAfterEnd,
    /// The display set contains no end segments (ES).
    #[error("display set contains no end segment")]
    MissingEndSegment,
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
    /// Reads the next display set from a source.
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

        DisplaySet::from_segments(segments)
    }
}

impl DisplaySet {

    pub fn from_segments<T>(value: T) -> ReadResult<Self> where
        T: IntoIterator<Item = Segment>
    {

        let mut es = None;
        let mut sequence = Sequence::Single;
        let mut initial_object = None;
        let mut middle_objects = Vec::new();
        let mut windows = BTreeMap::<u8, Window>::new();
        let mut palettes = BTreeMap::<Vid<u8>, Palette>::new();
        let mut objects = BTreeMap::<Vid<u16>, Object>::new();
        let mut composition_objects = BTreeMap::<Cid, CompositionObject>::new();
        let mut iterator = value.into_iter();
        let pcs = match iterator.next() {
            Some(segment) => {
                match segment {
                    Segment::PresentationComposition(pcs) => pcs,
                    _ => return Err(ReadError::MissingPresentationCompositionSegment),
                }
            }
            None => {
                return Err(ReadError::NoSegments)
            }
        };

        while let Some(segment) = iterator.next() {

            if es.is_some() {
                return Err(ReadError::SegmentAfterEnd)
            }

            match segment {
                Segment::PresentationComposition(_) => {
                    return Err(ReadError::UnexpectedPresentationCompositionSegment)
                }
                Segment::WindowDefinition(wds) => {
                    if wds.pts != pcs.pts {
                        return Err(ReadError::InconsistentPts)
                    }
                    if wds.dts != pcs.dts {
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
                Segment::PaletteDefinition(pds) => {
                    if pds.pts != pcs.pts {
                        return Err(ReadError::InconsistentPts)
                    }
                    if pds.dts != pcs.dts {
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
                Segment::SingleObjectDefinition(sods) => {
                    if sequence == Sequence::Single || sequence == Sequence::Final {
                        if sods.pts != pcs.pts {
                            return Err(ReadError::InconsistentPts)
                        }
                        if sods.dts != pcs.dts {
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
                Segment::InitialObjectDefinition(iods) => {
                    if sequence == Sequence::Single || sequence == Sequence::Final {
                        if iods.pts != pcs.pts {
                            return Err(ReadError::InconsistentPts)
                        }
                        if iods.dts != pcs.dts {
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
                Segment::MiddleObjectDefinition(mods) => {
                    if sequence == Sequence::Initial || sequence == Sequence::Middle {
                        match &initial_object {
                            Some(iods) => {
                                if mods.pts != pcs.pts {
                                    return Err(ReadError::InconsistentPts)
                                }
                                if mods.dts != pcs.dts {
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
                Segment::FinalObjectDefinition(mut fods) => {
                    if sequence == Sequence::Initial || sequence == Sequence::Middle {
                        match &mut initial_object {
                            Some(iods) => {
                                if fods.pts != pcs.pts {
                                    return Err(ReadError::InconsistentPts)
                                }
                                if fods.dts != pcs.dts {
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
                Segment::End(this_es) => {
                    if sequence != Sequence::Single && sequence != Sequence::Final {
                        return Err(ReadError::IncompleteObjectSequence)
                    }
                    if this_es.pts != pcs.pts {
                        return Err(ReadError::InconsistentPts)
                    }
                    if this_es.dts != pcs.dts {
                        return Err(ReadError::InconsistentDts)
                    }
                    es = Some(this_es);
                }
            }
        }

        if es.is_none() {
            return Err(ReadError::MissingEndSegment)
        }

        for co in pcs.composition_objects.iter() {
            composition_objects.insert(
                Cid {
                    object_id: co.object_id,
                    window_id: co.window_id,
                },
                CompositionObject {
                    x: co.x,
                    y: co.y,
                    forced: co.forced,
                    crop: co.crop.clone(),
                },
            );
        }

        let composition = Composition {
            number: pcs.composition_number,
            state: pcs.composition_state,
            objects: composition_objects,
        };

        match pcs.palette_update_id {
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
                pts: pcs.pts,
                dts: pcs.dts,
                width: pcs.width,
                height: pcs.height,
                frame_rate: pcs.frame_rate,
                palette_update_id: pcs.palette_update_id,
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
