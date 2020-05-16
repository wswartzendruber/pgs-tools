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

mod pgs {

    use std::{
        io::{Error as IoError, Read},
        result::Result,
    };
    use byteorder::{BigEndian, ReadBytesExt};
    use thiserror::Error as ThisError;

    pub type SegResult<T> = Result<T, SegError>;

    #[derive(ThisError, Debug)]
    pub enum SegError {
        #[error("segment IO error")]
        PrematureEof {
            #[from]
            source: IoError,
        },
        #[error("segment has unrecognized magic number")]
        UnrecognizedMagicNumber,
        #[error("segment has unrecognized kind")]
        UnrecognizedKind,
        #[error("presentation control segment has unrecognized frame rate")]
        UnrecognizedFrameRate,
        #[error("presentation control segment has unrecognized composition state")]
        UnrecognizedCompositionState,
        #[error("presentation control segment has unrecognized palette update flag")]
        UnrecognizedPaletteUpdateFlag,
        #[error("composition object has unrecognized cropped flag")]
        UnrecognizedCroppedFlag,
    }

    pub enum CompState {
        Normal,
        AcquisitionPoint,
        EpochStart,
    }

    pub struct PresCompSeg {
        width: u16,
        height: u16,
        comp_num: u16,
        comp_state: CompState,
        pal_update: bool,
        pal_id: u8,
        comp_objs: Vec<CompObj>,
    }

    pub struct CompObj {
        obj_id: u16,
        win_id: u8,
        x: u16,
        y: u16,
        crop: Option<CompObjCrop>,
    }

    pub struct CompObjCrop {
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    }

    pub enum SegBody {
        PresComp(PresCompSeg),
    }

    pub struct Seg {
        pts: u32,
        dts: u32,
        body: SegBody,
    }

    pub trait ReadExt {
        fn read_seg(&mut self) -> SegResult<Seg>;
    }

    impl ReadExt for dyn Read {

        fn read_seg(&mut self) -> SegResult<Seg> {

            if self.read_u16::<BigEndian>()? != 0x5047 {
                return Err(SegError::UnrecognizedMagicNumber)
            }

            let pts = self.read_u32::<BigEndian>()?;
            let dts = self.read_u32::<BigEndian>()?;
            let body = match self.read_u8()? {
                //0x14 => SegType::Pds,
                //0x15 => SegType::Ods,
                0x16 => SegBody::PresComp(parse_pcs(self)?),
                //0x17 => SegType::Wds,
                //0x80 => SegType::End,
                _ => return Err(SegError::UnrecognizedKind),
            };

            Ok(Seg { pts, dts, body })
        }
    }

    fn parse_pcs(input: &mut dyn Read) -> SegResult<PresCompSeg> {

        let size = input.read_u16::<BigEndian>()? as usize;
        let width = input.read_u16::<BigEndian>()?;
        let height = input.read_u16::<BigEndian>()?;

        if input.read_u8()? != 0x10 {
            return Err(SegError::UnrecognizedFrameRate)
        }

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

            let obj_id = input.read_u16::<BigEndian>()?;
            let win_id = input.read_u8()?;
            let cropped = match input.read_u8()? {
                0x40 => true,
                0x00 => false,
                _ => return Err(SegError::UnrecognizedCroppedFlag),
            };
            let x = input.read_u16::<BigEndian>()?;
            let y = input.read_u16::<BigEndian>()?;
            let crop = if cropped {
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
}
