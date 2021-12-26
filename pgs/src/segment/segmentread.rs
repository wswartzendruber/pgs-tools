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
    CompositionObject,
    Crop,
    CompositionState,
    EndSegment,
    ObjectDefinitionSegment,
    PaletteDefinitionSegment,
    PaletteEntry,
    PresentationCompositionSegment,
    Segment,
    Sequence,
    WindowDefinition,
    WindowDefinitionSegment,
};
use std::{
    io::{Cursor, Error as IoError, Read},
};
use byteorder::{BigEndian, ReadBytesExt};
use thiserror::Error as ThisError;

/// A specialized [`Result`](std::result::Result) type for segment-reading operations.
pub type ReadResult<T> = Result<T, ReadError>;

/// The error type for [ReadSegmentExt].
///
/// Errors are caused by either an invalid bitstream or by an underlying I/O error.
#[derive(ThisError, Debug)]
pub enum ReadError {
    /// The segment could not be read because of an underlying I/O error.
    #[error("segment IO error")]
    IoError {
        /// The underlying I/O error.
        #[from]
        source: IoError,
    },
    /// The bitstream declares an unrecognized magic number for the segment. This value should
    /// always be `0x5047`.
    #[error("segment has unrecognized magic number")]
    UnrecognizedMagicNumber {
        /// The magic number that was parsed.
        parsed_magic_number: u16,
    },
    /// The bitstream declares an unrecognized kind of segment. The valid kinds are:
    /// - `0x14` (PDS, or pallete definition segment)
    /// - `0x15` (ODS, or object definition segment)
    /// - `0x16` (PCS, or presentation composition segment)
    /// - `0x17` (WDS, or window definition segment)
    /// - `0x80` (ES, or end segment)
    #[error("segment has unrecognized kind")]
    UnrecognizedKind {
        /// The kind value that was parsed.
        parsed_kind: u8,
    },
    /// The bitstream declares an unrecognized composition state within a presentation
    /// composition segment (PCS). The valid states are:
    /// - `0x00` (maps to [`CompositionState::Normal`])
    /// - `0x40` (maps to [`CompositionState::AcquisitionPoint`])
    /// - `0x80` (maps to [`CompositionState::EpochStart`])
    #[error("presentation composition segment has unrecognized composition state")]
    UnrecognizedCompositionState {
        /// The composition state value that was parsed.
        parsed_composition_state: u8,
    },
    /// The bitstream declares an unrecognized palette update flag within a presentation
    /// composition segment (PCS). The valid flags are:
    /// - `0x00` (no palette updates are defined)
    /// - `0x80` (a preceding palette within the epoch will be updated)
    #[error("presentation composition segment has unrecognized palette update flag")]
    UnrecognizedPaletteUpdateFlag {
        /// The palette update flag that was parsed.
        parsed_palette_update_flag: u8,
    },
    /// The bitstream declares an invalid crop flag within a composition object within a
    /// presentation composition segment (PCS). The valid flags are:
    /// - `0x00` (no object cropping is being performed for this composition)
    /// - `0x40` (object cropping is being performed for this composition)
    #[error("composition object has unrecognized cropped flag")]
    UnrecognizedCropFlag {
        /// The crop flag that was parsed.
        parsed_crop_flag: u8,
    },
    /// The bitstream declares an unrecognized sequence flag within an object definition segment
    /// (ODS). The valid flags are:
    /// - `0xC0` (maps to [`Sequence::Single`])
    /// - `0x80` (maps to [`Sequence::First`])
    /// - `0x40` (maps to [`Sequence::Last`])
    #[error("unrecognized object definition sequence flag")]
    UnrecognizedObjectSequenceFlag {
        /// The sequence flag that was parsed.
        parsed_sequence_flag: u8,
    },
    /// The bitstream declares an invalid data length within an object definition segment (ODS).
    /// Specifically, the declared data length must agree with the segment's total size.
    #[error("invalid object data length")]
    InvalidObjectDataLength {
        /// The data length that was parsed.
        parsed_data_length: u32,
        /// The data length that was expected.
        expected_data_length: u32,
    },
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

/// Allows reading segments from a source.
pub trait ReadSegmentExt {
    /// Reads the next segment from a source.
    fn read_segment(&mut self) -> ReadResult<Segment>;
}

impl<T> ReadSegmentExt for T where
    T: Read,
{

    fn read_segment(&mut self) -> ReadResult<Segment> {

        let parsed_magic_number = self.read_u16::<BigEndian>()?;

        if parsed_magic_number != 0x5047 {
            return Err(ReadError::UnrecognizedMagicNumber { parsed_magic_number })
        }

        let pts = self.read_u32::<BigEndian>()?;
        let dts = self.read_u32::<BigEndian>()?;
        let parsed_kind = self.read_u8()?;
        let size = self.read_u16::<BigEndian>()? as usize;

        let mut payload = vec![0u8; size];
        self.read_exact(&mut payload)?;

        Ok(
            match parsed_kind {
                0x14 => Segment::PaletteDefinition(parse_pds(pts, dts, &payload)?),
                0x15 => Segment::ObjectDefinition(parse_ods(pts, dts, &payload)?),
                0x16 => Segment::PresentationComposition(parse_pcs(pts, dts, &payload)?),
                0x17 => Segment::WindowDefinition(parse_wds(pts, dts, &payload)?),
                0x80 => Segment::End(EndSegment { pts, dts }),
                _ => return Err(ReadError::UnrecognizedKind { parsed_kind }),
            }
        )
    }
}

fn parse_pcs(
    pts: u32,
    dts: u32,
    payload: &[u8],
) -> ReadResult<PresentationCompositionSegment> {

    let mut pos = 11;
    let mut input = Cursor::new(payload);
    let width = input.read_u16::<BigEndian>()?;
    let height = input.read_u16::<BigEndian>()?;
    let frame_rate = input.read_u8()?;

    let composition_number = input.read_u16::<BigEndian>()?;
    let parsed_composition_state = input.read_u8()?;
    let composition_state = match parsed_composition_state {
        0x00 => CompositionState::Normal,
        0x40 => CompositionState::AcquisitionPoint,
        0x80 => CompositionState::EpochStart,
        _ => return Err(ReadError::UnrecognizedCompositionState { parsed_composition_state }),
    };
    let parsed_palette_update_flag =input.read_u8()?;
    let palette_update_id = match parsed_palette_update_flag {
        0x00 => {
            input.read_u8()?;
            None
        }
        0x80 => {
            Some(input.read_u8()?)
        }
        _ => {
            return Err(ReadError::UnrecognizedPaletteUpdateFlag { parsed_palette_update_flag })
        }
    };
    let comp_obj_count = input.read_u8()? as usize;
    let mut composition_objects = Vec::new();

    for _ in 0..comp_obj_count {
        if payload.len() - pos >= 8 {

            let object_id = input.read_u16::<BigEndian>()?;
            let window_id = input.read_u8()?;
            let parsed_crop_flag = input.read_u8()?;
            let cropped = match parsed_crop_flag {
                0x40 => true,
                0x00 => false,
                _ => return Err(ReadError::UnrecognizedCropFlag { parsed_crop_flag }),
            };
            let x = input.read_u16::<BigEndian>()?;
            let y = input.read_u16::<BigEndian>()?;

            pos += 8;

            // For some reason, the U.S. release of Final Fantasy VII: Advent Children Complete
            // declares that the object is cropped, but then the segment's payload ends.
            let crop = if cropped && payload.len() - pos >= 8 {
                pos += 8;
                Some(
                    Crop {
                        x: input.read_u16::<BigEndian>()?,
                        y: input.read_u16::<BigEndian>()?,
                        width: input.read_u16::<BigEndian>()?,
                        height: input.read_u16::<BigEndian>()?,
                    }
                )
            } else {
                None
            };

            composition_objects.push(
                CompositionObject {
                    object_id,
                    window_id,
                    x,
                    y,
                    crop,
                }
            );
        }
    }

    Ok(
        PresentationCompositionSegment {
            pts,
            dts,
            width,
            height,
            frame_rate,
            composition_number,
            composition_state,
            palette_update_id,
            composition_objects,
        }
    )
}

fn parse_wds(
    pts: u32,
    dts: u32,
    payload: &[u8],
) -> ReadResult<WindowDefinitionSegment> {

    let mut input = Cursor::new(payload);
    let mut windows = Vec::new();
    let count = input.read_u8()?;

    for _ in 0..count {
        windows.push(
            WindowDefinition {
                id: input.read_u8()?,
                x: input.read_u16::<BigEndian>()?,
                y: input.read_u16::<BigEndian>()?,
                width: input.read_u16::<BigEndian>()?,
                height: input.read_u16::<BigEndian>()?,
            }
        );
    }

    Ok(
        WindowDefinitionSegment {
            pts,
            dts,
            windows,
        }
    )
}

fn parse_pds(
    pts: u32,
    dts: u32,
    payload: &[u8],
) -> ReadResult<PaletteDefinitionSegment> {

    let mut input = Cursor::new(payload);
    let count = (payload.len() - 2) / 5;
    let id = input.read_u8()?;
    let version = input.read_u8()?;
    let mut entries = Vec::new();

    for _ in 0..count {

        let id = input.read_u8()?;
        let y = input.read_u8()?;
        let cr = input.read_u8()?;
        let cb = input.read_u8()?;
        let alpha = input.read_u8()?;

        entries.push(PaletteEntry { id, y, cr, cb, alpha });
    }

    Ok(
        PaletteDefinitionSegment {
            pts,
            dts,
            id,
            version,
            entries,
        }
    )
}

fn parse_ods(
    pts: u32,
    dts: u32,
    payload: &[u8],
) -> ReadResult<ObjectDefinitionSegment> {

    let mut input = Cursor::new(&payload);
    let id = input.read_u16::<BigEndian>()?;
    let version = input.read_u8()?;
    let parsed_sequence_flag = input.read_u8()?;
    let sequence = match parsed_sequence_flag {
        0xC0 => Sequence::Single,
        0x80 => Sequence::First,
        0x40 => Sequence::Last,
        _ => return Err(ReadError::UnrecognizedObjectSequenceFlag { parsed_sequence_flag }),
    };

    // PGS streams record +4 bytes for the object data size, for some reason.
    let parsed_data_length = input.read_u24::<BigEndian>()?;
    let expected_data_length = (payload.len() - 7) as u32;

    if parsed_data_length != expected_data_length {
        return Err(
            ReadError::InvalidObjectDataLength {
                parsed_data_length,
                expected_data_length,
            }
        )
    }

    let width = input.read_u16::<BigEndian>()?;
    let height = input.read_u16::<BigEndian>()?;
    let lines = rle_decompress(&input.into_inner()[11..])?;

    Ok(
        ObjectDefinitionSegment {
            pts,
            dts,
            id,
            version,
            sequence,
            width,
            height,
            lines,
        }
    )
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
