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
use std::io::{
    Result as IoResult,
    Write,
};
use byteorder::{BigEndian, WriteBytesExt};

pub trait WriteSegExt {
    fn write_seg(&mut self, seg: &Seg) -> IoResult<()>;
}

impl<T> WriteSegExt for T where T: Write {

    fn write_seg(&mut self, seg: &Seg) -> IoResult<()> {

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

        self.write_u16::<BigEndian>(payload.len() as u16);
        self.write_all(&payload);

        Ok(())
    }
}

fn write_pcs(pcs: &PresCompSeg) -> IoResult<Vec<u8>> {

    Ok(vec![])
}

fn write_wds(wds: &[WinDefSeg]) -> IoResult<Vec<u8>> {

    Ok(vec![])
}

fn write_pds(pds: &PalDefSeg) -> IoResult<Vec<u8>> {

    Ok(vec![])
}

fn write_ods(ods: &ObjDefSeg) -> IoResult<Vec<u8>> {

    Ok(vec![])
}
