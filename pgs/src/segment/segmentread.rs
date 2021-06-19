/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: OSL-3.0
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

pub type SegmentReadResult<T> = Result<T, ReadError>;

#[derive(ThisError, Debug)]
pub enum ReadError {
    #[error("segment IO error")]
    IoError {
        #[from]
        source: IoError,
    },
    #[error("segment has unrecognized magic number")]
    UnrecognizedMagicNumber,
    #[error("segment has unrecognized kind")]
    UnrecognizedKind,
    #[error("presentation composition segment has unrecognized composition state")]
    UnrecognizedCompositionState,
    #[error("presentation composition segment has unrecognized palette update flag")]
    UnrecognizedPaletteUpdateFlag,
    #[error("composition object has unrecognized cropped flag")]
    UnrecognizedCropFlag,
    #[error("unrecognized object definition sequence flag")]
    UnrecognizedObjectSequenceFlag,
}

pub trait ReadSegmentExt {
    fn read_segment(&mut self) -> SegmentReadResult<Segment>;
}

impl<T: Read> ReadSegmentExt for T {

    fn read_segment(&mut self) -> SegmentReadResult<Segment> {

        if self.read_u16::<BigEndian>()? != 0x5047 {
            return Err(ReadError::UnrecognizedMagicNumber)
        }

        let pts = self.read_u32::<BigEndian>()?;
        let dts = self.read_u32::<BigEndian>()?;
        let kind = self.read_u8()?;
        let size = self.read_u16::<BigEndian>()? as usize;

        let mut payload = vec![0u8; size];
        self.read_exact(&mut payload)?;

        Ok(
            match kind {
                0x14 => Segment::PaletteDefinition(parse_pds(pts, dts, &payload)?),
                0x15 => Segment::ObjectDefinition(parse_ods(pts, dts, &payload)?),
                0x16 => Segment::PresentationComposition(parse_pcs(pts, dts, &payload)?),
                0x17 => Segment::WindowDefinition(parse_wds(pts, dts, &payload)?),
                0x80 => Segment::End(EndSegment { pts, dts }),
                _ => return Err(ReadError::UnrecognizedKind),
            }
        )
    }
}

fn parse_pcs(
    pts: u32,
    dts: u32,
    payload: &[u8],
) -> SegmentReadResult<PresentationCompositionSegment> {

    let mut pos = 11;
    let mut input = Cursor::new(payload);
    let width = input.read_u16::<BigEndian>()?;
    let height = input.read_u16::<BigEndian>()?;
    let frame_rate = input.read_u8()?;

    let composition_number = input.read_u16::<BigEndian>()?;
    let composition_state = match input.read_u8()? {
        0x00 => CompositionState::Normal,
        0x40 => CompositionState::AcquisitionPoint,
        0x80 => CompositionState::EpochStart,
        _ => return Err(ReadError::UnrecognizedCompositionState),
    };
    let palette_update_id = match input.read_u8()? {
        0x00 => {
            input.read_u8()?;
            None
        }
        0x80 => {
            Some(input.read_u8()?)
        }
        _ => {
            return Err(ReadError::UnrecognizedPaletteUpdateFlag)
        }
    };
    let comp_obj_count = input.read_u8()? as usize;
    let mut composition_objects = Vec::new();

    for _ in 0..comp_obj_count {
        if payload.len() - pos >= 8 {

            let object_id = input.read_u16::<BigEndian>()?;
            let window_id = input.read_u8()?;
            let cropped = match input.read_u8()? {
                0x40 => true,
                0x00 => false,
                _ => return Err(ReadError::UnrecognizedCropFlag),
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
) -> SegmentReadResult<WindowDefinitionSegment> {

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
) -> SegmentReadResult<PaletteDefinitionSegment> {

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
) -> SegmentReadResult<ObjectDefinitionSegment> {

    let mut input = Cursor::new(&payload);
    let id = input.read_u16::<BigEndian>()?;
    let version = input.read_u8()?;
    let sequence = match input.read_u8()? {
        0xC0 => Sequence::Single,
        0x80 => Sequence::First,
        0x40 => Sequence::Last,
        _ => return Err(ReadError::UnrecognizedObjectSequenceFlag),
    };
    let data_size = input.read_u24::<BigEndian>()? as usize;
    let width = input.read_u16::<BigEndian>()?;
    let height = input.read_u16::<BigEndian>()?;
    let mut data = vec![0u8; (data_size - 4).max(0)];

    input.read_exact(&mut data)?;

    Ok(
        ObjectDefinitionSegment {
            pts,
            dts,
            id,
            version,
            sequence,
            width,
            height,
            data,
        }
    )
}
