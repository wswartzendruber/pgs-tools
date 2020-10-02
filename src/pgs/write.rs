/*
 * Copyright 2020 William Swartzendruber
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 * except in compliance with the License. You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software distributed under the
 * License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
 * either express or implied. See the License for the specific language governing permissions
 * and limitations under the License.
 */

use super::{
    CompState,
    ObjDefSeg,
    ObjSeq,
    PalDefSeg,
    PresCompSeg,
    Seg,
    SegBody,
    WinDefSeg,
};
use std::io::{
    Error as IoError,
    Write,
};
use byteorder::{BigEndian, WriteBytesExt};
use thiserror::Error as ThisError;

pub type SegWriteResult<T> = Result<T, SegWriteError>;

#[derive(ThisError, Debug)]
pub enum SegWriteError {
    #[error("segment IO error")]
    IoError {
        #[from]
        source: IoError,
    },
    #[error("too many composition objects in presentation composition segment")]
    TooManyCompObjs,
    #[error("too many window definition segments")]
    TooManyWinDefSegs,
    #[error("too many palette entries")]
    TooManyPalEntries,
    #[error("object data too large")]
    ObjDataTooLarge,
}

pub trait WriteSegExt {
    fn write_seg(&mut self, seg: &Seg) -> SegWriteResult<()>;
}

impl<T> WriteSegExt for T where T: Write {

    fn write_seg(&mut self, seg: &Seg) -> SegWriteResult<()> {

        self.write_u16::<BigEndian>(0x5047)?;
        self.write_u32::<BigEndian>(seg.pts)?;
        self.write_u32::<BigEndian>(seg.dts)?;

        let payload = match &seg.body {
            SegBody::PresComp(pcs) => {
                self.write_u8(0x16)?;
                write_pcs(pcs)?
            },
            SegBody::WinDef(wds) => {
                self.write_u8(0x17)?;
                write_wds(wds)?
            },
            SegBody::PalDef(pds) => {
                self.write_u8(0x14)?;
                write_pds(pds)?
            },
            SegBody::ObjDef(ods) => {
                self.write_u8(0x15)?;
                write_ods(ods)?
            },
            SegBody::End(_) => {
                self.write_u8(0x80)?;
                vec![]
            },
        };

        self.write_u16::<BigEndian>(payload.len() as u16)?;
        self.write_all(&payload)?;

        Ok(())
    }
}

fn write_pcs(pcs: &PresCompSeg) -> SegWriteResult<Vec<u8>> {

    let mut payload = vec![];

    payload.write_u16::<BigEndian>(pcs.width)?;
    payload.write_u16::<BigEndian>(pcs.height)?;
    payload.write_u8(pcs.frame_rate)?;
    payload.write_u16::<BigEndian>(pcs.comp_num)?;
    payload.write_u8(
        match pcs.comp_state {
            CompState::Normal => 0x00,
            CompState::AcquisitionPoint => 0x40,
            CompState::EpochStart => 0x80,
        }
    )?;
    payload.write_u8(
        if pcs.pal_update {
            0x80
        } else {
            0x00
        }
    )?;
    payload.write_u8(pcs.pal_id)?;

    if pcs.comp_objs.len() <= 255 {
        payload.write_u8(pcs.comp_objs.len() as u8)?;
    } else {
        return Err(SegWriteError::TooManyCompObjs)
    }

    for comp_obj in &pcs.comp_objs {

        payload.write_u16::<BigEndian>(comp_obj.obj_id)?;
        payload.write_u8(comp_obj.win_id)?;

        let cropped = comp_obj.crop.is_some();

        payload.write_u8(
            if cropped {
                0x40
            } else {
                0x00
            }
        )?;
        payload.write_u16::<BigEndian>(comp_obj.x)?;
        payload.write_u16::<BigEndian>(comp_obj.y)?;

        if cropped {

            let crop = comp_obj.crop.as_ref().unwrap();

            payload.write_u16::<BigEndian>(crop.x)?;
            payload.write_u16::<BigEndian>(crop.y)?;
            payload.write_u16::<BigEndian>(crop.width)?;
            payload.write_u16::<BigEndian>(crop.height)?;
        }
    }

    Ok(payload)
}

fn write_wds(wds: &[WinDefSeg]) -> SegWriteResult<Vec<u8>> {

    let mut payload = vec![];

    if wds.len() <= 255 {
        payload.write_u8(wds.len() as u8)?;
    } else {
        return Err(SegWriteError::TooManyWinDefSegs)
    }

    for win in wds {
        payload.write_u8(win.id)?;
        payload.write_u16::<BigEndian>(win.x)?;
        payload.write_u16::<BigEndian>(win.y)?;
        payload.write_u16::<BigEndian>(win.width)?;
        payload.write_u16::<BigEndian>(win.height)?;
    }

    Ok(payload)
}

fn write_pds(pds: &PalDefSeg) -> SegWriteResult<Vec<u8>> {

    let mut payload = vec![];

    payload.write_u8(pds.id)?;
    payload.write_u8(pds.version)?;

    for entry in &pds.entries {
        payload.write_u8(entry.id)?;
        payload.write_u8(entry.y)?;
        payload.write_u8(entry.cr)?;
        payload.write_u8(entry.cb)?;
        payload.write_u8(entry.alpha)?;
    }

    Ok(payload)
}

fn write_ods(ods: &ObjDefSeg) -> SegWriteResult<Vec<u8>> {

    let mut payload = vec![];

    payload.write_u16::<BigEndian>(ods.id)?;
    payload.write_u8(ods.version)?;
    payload.write_u8(
        match &ods.seq {
            Some(seq) => match seq {
                ObjSeq::Last => 0x40,
                ObjSeq::First => 0x80,
                ObjSeq::Both => 0xC0,
            },
            None => 0x00,
        }
    )?;

    // I have no idea why PGS streams record +4 bytes for the object data size, but they do.
    if ods.data.len() <= 16_777_212 {
        payload.write_u24::<BigEndian>((ods.data.len() + 4) as u32)?;
    } else {
        return Err(SegWriteError::ObjDataTooLarge)
    }

    payload.write_u16::<BigEndian>(ods.width)?;
    payload.write_u16::<BigEndian>(ods.height)?;
    payload.write_all(&ods.data)?;

    Ok(payload)
}
