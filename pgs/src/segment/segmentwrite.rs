/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: OSL-3.0
 */

use super::{
    CompositionState,
    EndSegment,
    ObjectDefinitionSegment,
    ObjectSequence,
    PaletteDefinitionSegment,
    PresentationCompositionSegment,
    Segment,
    WindowDefinitionSegment,
};
use std::io::{
    Error as IoError,
    Write,
};
use byteorder::{BigEndian, WriteBytesExt};
use thiserror::Error as ThisError;

pub type SegmentWriteResult<T> = Result<T, WriteError>;

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
}

pub trait WriteSegmentExt {
    fn write_segment(&mut self, segment: &Segment) -> SegmentWriteResult<()>;
}

impl<T: Write> WriteSegmentExt for T {

    fn write_segment(&mut self, segment: &Segment) -> SegmentWriteResult<()> {

        self.write_u16::<BigEndian>(0x5047)?;

        let payload = match &segment {
            Segment::PresentationComposition(pcs) => generate_pcs(pcs)?,
            Segment::WindowDefinition(wds) => generate_wds(wds)?,
            Segment::PaletteDefinition(pds) => generate_pds(pds)?,
            Segment::ObjectDefinition(ods) => generate_ods(ods)?,
            Segment::End(es) => generate_es(es)?,
        };

        self.write_u16::<BigEndian>(payload.len() as u16)?;
        self.write_all(&payload)?;

        Ok(())
    }
}

fn generate_pcs(pcs: &PresentationCompositionSegment) -> SegmentWriteResult<Vec<u8>> {

    let mut payload = vec![];

    payload.write_u32::<BigEndian>(pcs.pts)?;
    payload.write_u32::<BigEndian>(pcs.dts)?;
    payload.write_u8(0x16)?;
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

fn generate_wds(wds: &WindowDefinitionSegment) -> SegmentWriteResult<Vec<u8>> {

    let mut payload = vec![];

    payload.write_u32::<BigEndian>(wds.pts)?;
    payload.write_u32::<BigEndian>(wds.dts)?;
    payload.write_u8(0x17)?;

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

fn generate_pds(pds: &PaletteDefinitionSegment) -> SegmentWriteResult<Vec<u8>> {

    let mut payload = vec![];

    payload.write_u32::<BigEndian>(pds.pts)?;
    payload.write_u32::<BigEndian>(pds.dts)?;
    payload.write_u8(0x14)?;
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

fn generate_ods(ods: &ObjectDefinitionSegment) -> SegmentWriteResult<Vec<u8>> {

    let mut payload = vec![];

    payload.write_u32::<BigEndian>(ods.pts)?;
    payload.write_u32::<BigEndian>(ods.dts)?;
    payload.write_u8(0x15)?;
    payload.write_u16::<BigEndian>(ods.id)?;
    payload.write_u8(ods.version)?;
    payload.write_u8(
        match &ods.sequence {
            Some(sequence) => match sequence {
                ObjectSequence::Last => 0x40,
                ObjectSequence::First => 0x80,
                ObjectSequence::Both => 0xC0,
            },
            None => 0x00,
        }
    )?;

    // I have no idea why PGS streams record +4 bytes for the object data size, but they do.
    if ods.data.len() <= 16_777_212 {
        payload.write_u24::<BigEndian>((ods.data.len() + 4) as u32)?;
    } else {
        return Err(WriteError::ObjectDataTooLarge)
    }

    payload.write_u16::<BigEndian>(ods.width)?;
    payload.write_u16::<BigEndian>(ods.height)?;
    payload.write_all(&ods.data)?;

    Ok(payload)
}

fn generate_es(es: &EndSegment) -> SegmentWriteResult<Vec<u8>> {

    let mut payload = vec![];

    payload.write_u32::<BigEndian>(es.pts)?;
    payload.write_u32::<BigEndian>(es.dts)?;
    payload.write_u8(0x80)?;

    Ok(payload)
}
