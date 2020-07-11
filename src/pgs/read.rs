/*
 * Copyright 2020 William Swartzendruber
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a
 * copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use super::{
    CompObj,
    CompObjCrop,
    CompState,
    EndSeg,
    ObjDefSeg,
    ObjSeq,
    PalDefSeg,
    PalEntry,
    PresCompSeg,
    Seg,
    SegBody,
    WinDefSeg,
};
use std::{
    cmp::min,
    io::{Cursor, Error as IoError, Read},
};
use byteorder::{BigEndian, ReadBytesExt};
use thiserror::Error as ThisError;

pub type SegReadResult<T> = Result<T, SegReadError>;

#[derive(ThisError, Debug)]
pub enum SegReadError {
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
    UnrecognizedCompState,
    #[error("presentation composition segment has unrecognized palette update flag")]
    UnrecognizedPalUpdateFlag,
    #[error("composition object has unrecognized cropped flag")]
    UnrecognizedCroppedFlag,
    #[error("unrecognized object definition sequence flag")]
    UnrecognizedObjSeqFlag,
}

pub trait ReadSegExt {
    fn read_seg(&mut self) -> SegReadResult<Seg>;
}

impl<T> ReadSegExt for T where T: Read  {

    fn read_seg(&mut self) -> SegReadResult<Seg> {

        if self.read_u16::<BigEndian>()? != 0x5047 {
            return Err(SegReadError::UnrecognizedMagicNumber)
        }

        let pts = self.read_u32::<BigEndian>()?;
        let dts = self.read_u32::<BigEndian>()?;
        let kind = self.read_u8()?;
        let size = self.read_u16::<BigEndian>()? as usize;

        let mut payload = vec![0u8; size];
        self.read_exact(&mut payload)?;

        let body = match kind {
            0x14 => SegBody::PalDef(parse_pds(&payload)?),
            0x15 => SegBody::ObjDef(parse_ods(&payload)?),
            0x16 => SegBody::PresComp(parse_pcs(&payload)?),
            0x17 => SegBody::WinDef(parse_wds(&payload)?),
            0x80 => SegBody::End(EndSeg { }),
            _ => return Err(SegReadError::UnrecognizedKind),
        };

        Ok(Seg { pts, dts, body })
    }
}

fn parse_pcs(payload: &[u8]) -> SegReadResult<PresCompSeg> {

    let mut pos = 11;
    let mut input = Cursor::new(payload);
    let width = input.read_u16::<BigEndian>()?;
    let height = input.read_u16::<BigEndian>()?;
    let frame_rate = input.read_u8()?;

    let comp_num = input.read_u16::<BigEndian>()?;
    let comp_state = match input.read_u8()? {
        0x00 => CompState::Normal,
        0x40 => CompState::AcquisitionPoint,
        0x80 => CompState::EpochStart,
        _ => return Err(SegReadError::UnrecognizedCompState),
    };
    let pal_update = match input.read_u8()? {
        0x00 => false,
        0x80 => true,
        _ => return Err(SegReadError::UnrecognizedPalUpdateFlag),
    };
    let pal_id = input.read_u8()?;
    let comp_obj_count = input.read_u8()? as usize;
    let mut comp_objs = Vec::new();

    for _ in 0..comp_obj_count {
        if payload.len() - pos >= 8 {

            let obj_id = input.read_u16::<BigEndian>()?;
            let win_id = input.read_u8()?;
            let cropped = match input.read_u8()? {
                0x40 => true,
                0x00 => false,
                _ => return Err(SegReadError::UnrecognizedCroppedFlag),
            };
            let x = input.read_u16::<BigEndian>()?;
            let y = input.read_u16::<BigEndian>()?;

            pos += 8;

            // For some reason, the U.S. release of Final Fantasy VII: Advent Children Complete
            // declares that the object is cropped, but then the segment's payload ends.
            let crop = if cropped && payload.len() - pos >= 8 {
                pos += 8;
                Some(
                    CompObjCrop {
                        x: input.read_u16::<BigEndian>()?,
                        y: input.read_u16::<BigEndian>()?,
                        width: input.read_u16::<BigEndian>()?,
                        height: input.read_u16::<BigEndian>()?,
                    }
                )
            } else {
                None
            };

            comp_objs.push(
                CompObj {
                    obj_id,
                    win_id,
                    x,
                    y,
                    crop,
                }
            );
        }
    }

    Ok(
        PresCompSeg {
            width,
            height,
            frame_rate,
            comp_num,
            comp_state,
            pal_update,
            pal_id,
            comp_objs,
        }
    )
}

fn parse_wds(payload: &[u8]) -> SegReadResult<Vec<WinDefSeg>> {

    let mut input = Cursor::new(payload);
    let mut return_value = Vec::new();
    let count = input.read_u8()?;

    for _ in 0..count {

        let id = input.read_u8()?;
        let x = input.read_u16::<BigEndian>()?;
        let y = input.read_u16::<BigEndian>()?;
        let width = input.read_u16::<BigEndian>()?;
        let height = input.read_u16::<BigEndian>()?;

        return_value.push(WinDefSeg { id, x, y, width, height });
    }

    Ok(return_value)
}

fn parse_pds(payload: &[u8]) -> SegReadResult<PalDefSeg> {

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

        entries.push(PalEntry { id, y, cr, cb, alpha });
    }

    Ok(PalDefSeg { id, version, entries })
}

fn parse_ods(payload: &[u8]) -> SegReadResult<ObjDefSeg> {

    let mut input = Cursor::new(&payload);
    let id = input.read_u16::<BigEndian>()?;
    let version = input.read_u8()?;
    let seq = match input.read_u8()? {
        0x00 => None,
        0x40 => Some(ObjSeq::Last),
        0x80 => Some(ObjSeq::First),
        0xC0 => Some(ObjSeq::Both),
        _ => return Err(SegReadError::UnrecognizedObjSeqFlag),
    };
    let data_size = input.read_u24::<BigEndian>()? as usize;
    let width = input.read_u16::<BigEndian>()?;
    let height = input.read_u16::<BigEndian>()?;
    let mut data = vec![0u8; (data_size - 4).max(0)];

    input.read_exact(&mut data)?;

    Ok(ObjDefSeg { id, version, seq, width, height, data })
}
