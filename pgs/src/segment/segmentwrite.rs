/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: OSL-3.0
 */

use super::{
    CompositionState,
    ObjectDefinitionSegment,
    PaletteDefinitionSegment,
    PresentationCompositionSegment,
    Segment,
    Sequence,
    WindowDefinitionSegment,
};
use std::io::{
    Error as IoError,
    Write,
};
use byteorder::{BigEndian, WriteBytesExt};
use thiserror::Error as ThisError;

pub type WriteResult<T> = Result<T, WriteError>;

#[derive(ThisError, Debug)]
pub enum WriteError {
    #[error("segment IO error")]
    IoError {
        #[from]
        source: IoError,
    },
    #[error("too many composition objects in presentation composition segment")]
    TooManyCompositionObjects,
    #[error("too many window definitions")]
    TooManyWindowDefinitions,
    #[error("object data is too large")]
    ObjectDataTooLarge,
    #[error("object line too long")]
    ObjectLineTooLong,
}

pub trait WriteSegmentExt {
    fn write_segment(&mut self, segment: &Segment) -> WriteResult<()>;
}

impl<T: Write> WriteSegmentExt for T {

    fn write_segment(&mut self, segment: &Segment) -> WriteResult<()> {

        self.write_u16::<BigEndian>(0x5047)?;

        let payload = match &segment {
            Segment::PresentationComposition(pcs) => {
                self.write_u32::<BigEndian>(pcs.pts)?;
                self.write_u32::<BigEndian>(pcs.dts)?;
                self.write_u8(0x16)?;
                generate_pcs(pcs)?
            }
            Segment::WindowDefinition(wds) => {
                self.write_u32::<BigEndian>(wds.pts)?;
                self.write_u32::<BigEndian>(wds.dts)?;
                self.write_u8(0x17)?;
                generate_wds(wds)?
            }
            Segment::PaletteDefinition(pds) => {
                self.write_u32::<BigEndian>(pds.pts)?;
                self.write_u32::<BigEndian>(pds.dts)?;
                self.write_u8(0x14)?;
                generate_pds(pds)?
            }
            Segment::ObjectDefinition(ods) => {
                self.write_u32::<BigEndian>(ods.pts)?;
                self.write_u32::<BigEndian>(ods.dts)?;
                self.write_u8(0x15)?;
                generate_ods(ods)?
            }
            Segment::End(es) => {
                self.write_u32::<BigEndian>(es.pts)?;
                self.write_u32::<BigEndian>(es.dts)?;
                self.write_u8(0x80)?;
                vec![]
            }
        };

        self.write_u16::<BigEndian>(payload.len() as u16)?;
        self.write_all(&payload)?;

        Ok(())
    }
}

fn generate_pcs(pcs: &PresentationCompositionSegment) -> WriteResult<Vec<u8>> {

    let mut payload = vec![];

    payload.write_u16::<BigEndian>(pcs.width)?;
    payload.write_u16::<BigEndian>(pcs.height)?;
    payload.write_u8(pcs.frame_rate)?;
    payload.write_u16::<BigEndian>(pcs.composition_number)?;
    payload.write_u8(
        match pcs.composition_state {
            CompositionState::Normal => 0x00,
            CompositionState::AcquisitionPoint => 0x40,
            CompositionState::EpochStart => 0x80,
        }
    )?;

    match pcs.palette_update_id {
        Some(pal_id) => {
            payload.write_u8(0x80)?;
            payload.write_u8(pal_id)?;
        }
        None => {
            payload.write_u8(0x00)?;
            payload.write_u8(0)?;
        }
    }

    if pcs.composition_objects.len() <= 255 {
        payload.write_u8(pcs.composition_objects.len() as u8)?;
    } else {
        return Err(WriteError::TooManyCompositionObjects)
    }

    for comp_obj in &pcs.composition_objects {

        payload.write_u16::<BigEndian>(comp_obj.object_id)?;
        payload.write_u8(comp_obj.window_id)?;

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

fn generate_wds(wds: &WindowDefinitionSegment) -> WriteResult<Vec<u8>> {

    let mut payload = vec![];

    if wds.windows.len() <= 255 {
        payload.write_u8(wds.windows.len() as u8)?;
    } else {
        return Err(WriteError::TooManyWindowDefinitions)
    }

    for window in wds.windows.iter() {
        payload.write_u8(window.id)?;
        payload.write_u16::<BigEndian>(window.x)?;
        payload.write_u16::<BigEndian>(window.y)?;
        payload.write_u16::<BigEndian>(window.width)?;
        payload.write_u16::<BigEndian>(window.height)?;
    }

    Ok(payload)
}

fn generate_pds(pds: &PaletteDefinitionSegment) -> WriteResult<Vec<u8>> {

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

fn generate_ods(ods: &ObjectDefinitionSegment) -> WriteResult<Vec<u8>> {

    let mut payload = vec![];
    let data = rle_compress(&ods.lines)?;

    payload.write_u16::<BigEndian>(ods.id)?;
    payload.write_u8(ods.version)?;
    payload.write_u8(
        match &ods.sequence {
            Sequence::Single => 0xC0,
            Sequence::First => 0x80,
            Sequence::Last => 0x40,
        }
    )?;

    // I have no idea why PGS streams record +4 bytes for the object data size, but they do.
    if data.len() <= 16_777_211 {
        payload.write_u24::<BigEndian>((data.len() + 4) as u32)?;
    } else {
        return Err(WriteError::ObjectDataTooLarge)
    }

    payload.write_u16::<BigEndian>(ods.width)?;
    payload.write_u16::<BigEndian>(ods.height)?;
    payload.write_all(&data)?;

    Ok(payload)
}

fn rle_compress(input: &Vec<Vec<u8>>) -> WriteResult<Vec<u8>> {

    let mut output = Vec::<u8>::new();
    let mut byte = 0_u8;
    let mut count = 0_usize;

    for line in input.iter() {

        for next_byte in line.iter() {
            if *next_byte == byte {
                count += 1;
            } else {
                if count > 0 {
                    output_rle_sequence(&mut output, byte, count)?;
                }
                byte = *next_byte;
                count = 1;
            }
        }

        output_rle_sequence(&mut output, byte, count)?;
        byte = 0;
        count = 0;

        output.push(0x00);
        output.push(0x00);
    }

    Ok(output)
}

fn output_rle_sequence(output: &mut Vec<u8>, byte: u8, count: usize) -> WriteResult<()> {

    if byte == 0x00 {
        match count {
            0 => {
                //panic!("attempted to handle zero-byte sequence in PGS line")
            }
            1 ..= 63 => {
                output.push(0x00);
                output.push(count as u8);
            }
            64 ..= 16383 => {
                output.push(0x00);
                output.push(0x40 | (count >> 8) as u8);
                output.push((count & 0xFF) as u8);
            }
            _ => {
                return Err(WriteError::ObjectLineTooLong)
            }
        }
    } else {
        match count {
            0 => {
                //panic!("attempted to handle zero-byte sequence in PGS line")
            }
            1 => {
                output.push(byte);
            }
            2 => {
                output.push(byte);
                output.push(byte);
            }
            3 ..= 63 => {
                output.push(0x00);
                output.push(0x80 | count as u8);
                output.push(byte);
            }
            64 ..= 16383 => {
                output.push(0x00);
                output.push(0xC0 | (count >> 8) as u8);
                output.push((count & 0xFF) as u8);
                output.push(byte);
            }
            _ => {
                return Err(WriteError::ObjectLineTooLong)
            }
        }
    }

    Ok(())
}
