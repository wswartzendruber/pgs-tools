/*
 * Copyright 2022 William Swartzendruber
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a
 * copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use super::{
    CompositionObject,
    Crop,
    CompositionState,
    EndSegment,
    FinalObjectDefinitionSegment,
    InitialObjectDefinitionSegment,
    MiddleObjectDefinitionSegment,
    PaletteDefinitionSegment,
    PaletteEntry,
    PresentationCompositionSegment,
    Segment,
    SingleObjectDefinitionSegment,
    WindowDefinition,
    WindowDefinitionSegment,
};
use std::{
    io::{Error as IoError, Read},
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
    /// The bitstream declares an unrecognized sequence flag within an object definition segment
    /// (ODS). The valid flags are:
    /// - `0xC0` (declares a single, complete object)
    /// - `0x80` (declares the initial portion of an object)
    /// - `0x40` (declares the final portion of an object)
    /// Otherwise, the segment is interpreted as being a middle portion.
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

        let magic_number = self.read_u16::<BigEndian>()?;

        if magic_number != 0x5047 {
            return Err(ReadError::UnrecognizedMagicNumber { parsed_magic_number: magic_number })
        }

        let pts = self.read_u32::<BigEndian>()?;
        let dts = self.read_u32::<BigEndian>()?;
        let kind = self.read_u8()?;
        let size = self.read_u16::<BigEndian>()?;

        Ok(
            match kind {
                0x14 => {
                    Segment::PaletteDefinition(parse_pds(pts, dts, self, size)?)
                }
                0x15 => {

                    let id = self.read_u16::<BigEndian>()?;
                    let version = self.read_u8()?;
                    let sequence_flag = self.read_u8()?;

                    match sequence_flag {
                        0xC0 => {
                            Segment::SingleObjectDefinition(
                                parse_sods(pts, dts, id, version, self, size)?
                            )
                        }
                        0x80 => {
                            Segment::InitialObjectDefinition(
                                parse_iods(pts, dts, id, version, self, size)?
                            )
                        }
                        0x00 => {
                            Segment::MiddleObjectDefinition(
                                parse_mods(pts, dts, id, version, self, size)?
                            )
                        }
                        0x40 => {
                            Segment::FinalObjectDefinition(
                                parse_fods(pts, dts, id, version, self, size)?
                            )
                        }
                        _ => {
                            return Err(
                                ReadError::UnrecognizedObjectSequenceFlag {
                                    parsed_sequence_flag: sequence_flag
                                }
                            )
                        }
                    }
                }
                0x16 => {
                    Segment::PresentationComposition(parse_pcs(pts, dts, self)?)
                }
                0x17 => {
                    Segment::WindowDefinition(parse_wds(pts, dts, self)?)
                }
                0x80 => {
                    Segment::End(EndSegment { pts, dts })
                }
                _ => {
                    return Err(ReadError::UnrecognizedKind { parsed_kind: kind })
                }
            }
        )
    }
}

fn parse_pcs(
    pts: u32,
    dts: u32,
    input: &mut dyn Read,
) -> ReadResult<PresentationCompositionSegment> {

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
    let palette_update_only = match parsed_palette_update_flag {
        0x00 => {
            false
        }
        0x80 => {
            true
        }
        _ => {
            return Err(ReadError::UnrecognizedPaletteUpdateFlag { parsed_palette_update_flag })
        }
    };
    let palette_id = input.read_u8()?;
    let comp_obj_count = input.read_u8()? as usize;
    let mut composition_objects = Vec::new();

    for _ in 0..comp_obj_count {

        let object_id = input.read_u16::<BigEndian>()?;
        let window_id = input.read_u8()?;
        let flags = input.read_u8()?;
        let x = input.read_u16::<BigEndian>()?;
        let y = input.read_u16::<BigEndian>()?;
        let forced = flags & 0x40 != 0;
        let crop = if flags & 0x80 != 0 {
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
                forced,
                crop,
            }
        );
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
            palette_update_only,
            palette_id,
            composition_objects,
        }
    )
}

fn parse_wds(
    pts: u32,
    dts: u32,
    input: &mut dyn Read,
) -> ReadResult<WindowDefinitionSegment> {

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
    input: &mut dyn Read,
    size: u16,
) -> ReadResult<PaletteDefinitionSegment> {

    let count = (size - 2) / 5;
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

fn parse_sods(
    pts: u32,
    dts: u32,
    id: u16,
    version: u8,
    input: &mut dyn Read,
    size: u16,
) -> ReadResult<SingleObjectDefinitionSegment> {

    // PGS streams record +4 bytes for the object data size, for some reason.
    let parsed_data_length = input.read_u24::<BigEndian>()?;
    let expected_data_length = size as u32 - 7;

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
    let mut data = vec![0x00_u8; size as usize - 11]; input.read_exact(&mut data)?;

    Ok(
        SingleObjectDefinitionSegment {
            pts,
            dts,
            id,
            version,
            width,
            height,
            data,
        }
    )
}

fn parse_iods(
    pts: u32,
    dts: u32,
    id: u16,
    version: u8,
    input: &mut dyn Read,
    size: u16,
) -> ReadResult<InitialObjectDefinitionSegment> {

    let length = input.read_u24::<BigEndian>()? as usize;
    let width = input.read_u16::<BigEndian>()?;
    let height = input.read_u16::<BigEndian>()?;
    let mut data = vec![0x00_u8; size as usize - 11]; input.read_exact(&mut data)?;

    Ok(
        InitialObjectDefinitionSegment {
            pts,
            dts,
            id,
            version,
            length,
            width,
            height,
            data,
        }
    )
}

fn parse_mods(
    pts: u32,
    dts: u32,
    id: u16,
    version: u8,
    input: &mut dyn Read,
    size: u16,
) -> ReadResult<MiddleObjectDefinitionSegment> {

    let mut data = vec![0x00_u8; size as usize - 4]; input.read_exact(&mut data)?;

    Ok(
        MiddleObjectDefinitionSegment {
            pts,
            dts,
            id,
            version,
            data,
        }
    )
}

fn parse_fods(
    pts: u32,
    dts: u32,
    id: u16,
    version: u8,
    input: &mut dyn Read,
    size: u16,
) -> ReadResult<FinalObjectDefinitionSegment> {

    let mut data = vec![0x00_u8; size as usize - 4]; input.read_exact(&mut data)?;

    Ok(
        FinalObjectDefinitionSegment {
            pts,
            dts,
            id,
            version,
            data,
        }
    )
}
