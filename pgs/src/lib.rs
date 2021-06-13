/*
 * Copyright © 2021 William Swartzendruber
 * Licensed under the Open Software License version 3.0
 */

pub mod read;
pub mod write;

pub enum SegBody {
    PresComp(PresCompSeg),
    WinDef(Vec<WinDefSeg>),
    PalDef(PalDefSeg),
    ObjDef(ObjDefSeg),
    End,
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
    pub frame_rate: u8,
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
