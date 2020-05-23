/*
 * Copyright 2020 William Swartzendruber
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of this software
 * and associated documentation files (the "Software"), to deal in the Software without
 * restriction, including without limitation the rights to use, copy, modify, merge, publish,
 * distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all copies or
 * substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
 * BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
 * DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

/*
 * WARNING: This code contains some disgusting hacks. This has been done because I have
 *          discovered that not all PGS bitstreams are compliant with what the patent states.
 *          The U.S. release of Final Fantasy VII: Advent Children Complete is particularly bad
 *          in this regard. So the parsing logic is written from the standpoint that the PGS
 *          bitstream has been obfuscated on purpose to make reading difficult.
 */

use std::{
    cmp::min,
    io::{Cursor, Error as IoError, Read},
    result::Result,
};
use byteorder::{BigEndian, ReadBytesExt};
use thiserror::Error as ThisError;

pub type SegResult<T> = Result<T, SegError>;

#[derive(ThisError, Debug)]
pub enum SegError {
    #[error("segment IO error")]
    IoError {
        #[from]
        source: IoError,
    },
    #[error("segment has unrecognized magic number")]
    UnrecognizedMagicNumber,
    #[error("segment has unrecognized kind")]
    UnrecognizedKind,
    #[error("presentation control segment has unrecognized composition state")]
    UnrecognizedCompositionState,
    #[error("presentation control segment has unrecognized palette update flag")]
    UnrecognizedPaletteUpdateFlag,
    #[error("composition object has unrecognized cropped flag")]
    UnrecognizedCroppedFlag,
    #[error("unrecognized object definition sequence flag")]
    UnrecognizedObjectSequenceFlag,
}

pub enum SegBody {
    PresComp(PresCompSeg),
    WinDef(Vec<WinDefSeg>),
    PalDef(PalDefSeg),
    ObjDef(ObjDefSeg),
    End(EndSeg),
}

pub enum CompState {
    Normal,
    AcquisitionPoint,
    EpochStart,
}

pub enum ObjSeq {
    Last,
    First,
    Both,
}

pub struct Seg {
    pub pts: u32,
    pub dts: u32,
    pub body: SegBody,
}

pub struct PresCompSeg {
    pub width: u16,
    pub height: u16,
    pub comp_num: u16,
    pub comp_state: CompState,
    pub pal_update: bool,
    pub pal_id: u8,
    pub comp_objs: Vec<CompObj>,
}

pub struct CompObj {
    pub obj_id: u16,
    pub win_id: u8,
    pub x: u16,
    pub y: u16,
    pub crop: Option<CompObjCrop>,
}

pub struct CompObjCrop {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

pub struct WinDefSeg {
    pub id: u8,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

pub struct PalDefSeg {
    pub id: u8,
    pub version: u8,
    pub entries: Vec<PalEntry>,
}

pub struct PalEntry {
    pub id: u8,
    pub y: u8,
    pub cr: u8,
    pub cb: u8,
    pub alpha: u8,
}

pub struct ObjDefSeg {
    pub id: u16,
    pub version: u8,
    pub seq: Option<ObjSeq>,
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

pub struct EndSeg { }

pub trait ReadSegExt {
    fn read_seg(&mut self) -> SegResult<Seg>;
}

impl<T> ReadSegExt for T where T: Read  {

    fn read_seg(&mut self) -> SegResult<Seg> {

        if self.read_u16::<BigEndian>()? != 0x5047 {
            return Err(SegError::UnrecognizedMagicNumber)
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
            _ => return Err(SegError::UnrecognizedKind),
        };

        Ok(Seg { pts, dts, body })
    }
}

fn parse_pcs(payload: &[u8]) -> SegResult<PresCompSeg> {

    let mut pos = 11;
    let mut input = Cursor::new(payload);
    let width = input.read_u16::<BigEndian>()?;
    let height = input.read_u16::<BigEndian>()?;

    // We ignore the frame rate; it could be full of crap.
    input.read_u8()?;

    let comp_num = input.read_u16::<BigEndian>()?;
    let comp_state = match input.read_u8()? {
        0x00 => CompState::Normal,
        0x40 => CompState::AcquisitionPoint,
        0x80 => CompState::EpochStart,
        _ => return Err(SegError::UnrecognizedCompositionState),
    };
    let pal_update = match input.read_u8()? {
        0x00 => false,
        0x80 => true,
        _ => return Err(SegError::UnrecognizedPaletteUpdateFlag),
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
                _ => return Err(SegError::UnrecognizedCroppedFlag),
            };
            let x = input.read_u16::<BigEndian>()?;
            let y = input.read_u16::<BigEndian>()?;

            pos += 8;

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
            comp_num,
            comp_state,
            pal_update,
            pal_id,
            comp_objs,
        }
    )
}

fn parse_wds(payload: &[u8]) -> SegResult<Vec<WinDefSeg>> {

    let mut input = Cursor::new(payload);
    let mut return_value = Vec::new();
    let count = min(input.read_u8()? as usize, (payload.len() - 1) % 9);

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

fn parse_pds(payload: &[u8]) -> SegResult<PalDefSeg> {

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

fn parse_ods(payload: &[u8]) -> SegResult<ObjDefSeg> {

    let mut input = Cursor::new(&payload);
    let id = input.read_u16::<BigEndian>()?;
    let version = input.read_u8()?;
    let seq = match input.read_u8()? {
        0x00 => None,
        0x40 => Some(ObjSeq::Last),
        0x80 => Some(ObjSeq::First),
        0xC0 => Some(ObjSeq::Both),
        _ => return Err(SegError::UnrecognizedObjectSequenceFlag),
    };
    let data_size = input.read_u24::<BigEndian>()? as usize;
    let width = input.read_u16::<BigEndian>()?;
    let height = input.read_u16::<BigEndian>()?;
    let mut data = vec![0u8; min(data_size, payload.len() - 11)];

    input.read_exact(&mut data)?;

    Ok(ObjDefSeg { id, version, seq, width, height, data })
}
