/*
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a
 * copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2022 William Swartzendruber
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use super::{
    CompositionState,
    Crop,
    FinalObjectDefinitionSegment,
    InitialObjectDefinitionSegment,
    MiddleObjectDefinitionSegment,
    PaletteDefinitionSegment,
    PresentationCompositionSegment,
    Segment,
    SingleObjectDefinitionSegment,
    WindowDefinitionSegment,
};
use std::io::{
    Error as IoError,
    Write,
};
use byteorder::{BigEndian, WriteBytesExt};
use thiserror::Error as ThisError;

/// A specialized [`Result`](std::result::Result) type for segment-writing operations.
pub type WriteResult<T> = Result<T, WriteError>;

/// The error type for [WriteSegmentExt].
///
/// Errors are caused by either invalid state or by an underlying I/O error.
#[derive(ThisError, Debug)]
pub enum WriteError {
    /// The [`Segment`] could not be written because of an underlying I/O error.
    #[error("segment IO error")]
    IoError {
        /// The underlying I/O error.
        #[from]
        source: IoError,
    },
    /// The [`Segment`] ([`PresentationCompositionSegment`]) being written has more than 255
    /// composition objects.
    #[error("too many composition objects in presentation composition segment")]
    TooManyCompositionObjects,
    /// The [`Segment`] ([`WindowDefinitionSegment`]) being written has more than 255 window
    /// definitions.
    #[error("too many window definitions")]
    TooManyWindowDefinitions,
    /// The [`Segment`] ([`SingleObjectDefinitionSegment`], [`InitialObjectDefinitionSegment`],
    /// [`MiddleObjectDefinitionSegment`], [`FinalObjectDefinitionSegment`]) being written has
    /// too much compressed data.
    #[error("object data is too large")]
    ObjectDataTooLarge,
}

/// Allows writing segments to a destination.
pub trait WriteSegmentExt {
    /// Writes a segment to a destination.
    fn write_segment(&mut self, segment: &Segment) -> WriteResult<()>;
}

impl<T> WriteSegmentExt for T where
    T: Write,
{

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
            Segment::SingleObjectDefinition(sods) => {
                self.write_u32::<BigEndian>(sods.pts)?;
                self.write_u32::<BigEndian>(sods.dts)?;
                self.write_u8(0x15)?;
                generate_sods(sods)?
            }
            Segment::InitialObjectDefinition(iods) => {
                self.write_u32::<BigEndian>(iods.pts)?;
                self.write_u32::<BigEndian>(iods.dts)?;
                self.write_u8(0x15)?;
                generate_iods(iods)?
            }
            Segment::MiddleObjectDefinition(mods) => {
                self.write_u32::<BigEndian>(mods.pts)?;
                self.write_u32::<BigEndian>(mods.dts)?;
                self.write_u8(0x15)?;
                generate_mods(mods)?
            }
            Segment::FinalObjectDefinition(fods) => {
                self.write_u32::<BigEndian>(fods.pts)?;
                self.write_u32::<BigEndian>(fods.dts)?;
                self.write_u8(0x15)?;
                generate_fods(fods)?
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
        payload.write_u8(match &comp_obj.crop {
            Crop::None => 0x00,
            Crop::Implicit => 0x40,
            Crop::Explicit { x: _, y: _, width: _, height: _ } => 0x80,
        })?;
        payload.write_u16::<BigEndian>(comp_obj.x)?;
        payload.write_u16::<BigEndian>(comp_obj.y)?;

        if let Crop::Explicit { x, y, width, height } = &comp_obj.crop {
            payload.write_u16::<BigEndian>(*x)?;
            payload.write_u16::<BigEndian>(*y)?;
            payload.write_u16::<BigEndian>(*width)?;
            payload.write_u16::<BigEndian>(*height)?;
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

fn generate_sods(ods: &SingleObjectDefinitionSegment) -> WriteResult<Vec<u8>> {

    // TODO: Validate data size (16-bit).

    let mut payload = vec![];

    payload.write_u16::<BigEndian>(ods.id)?;
    payload.write_u8(ods.version)?;
    payload.write_u8(0xC0)?;

    if ods.data.len() <= 16_777_211 {
        payload.write_u24::<BigEndian>(ods.data.len() as u32 + 4)?;
    } else {
        return Err(WriteError::ObjectDataTooLarge)
    }

    payload.write_u16::<BigEndian>(ods.width)?;
    payload.write_u16::<BigEndian>(ods.height)?;
    payload.write_all(&ods.data)?;

    Ok(payload)
}

fn generate_iods(ods: &InitialObjectDefinitionSegment) -> WriteResult<Vec<u8>> {

    // TODO: Validate data size (16-bit).

    let mut payload = vec![];

    payload.write_u16::<BigEndian>(ods.id)?;
    payload.write_u8(ods.version)?;
    payload.write_u8(0x80)?;

    if ods.length <= 16_777_215 {
        payload.write_u24::<BigEndian>(ods.length as u32)?;
    } else {
        return Err(WriteError::ObjectDataTooLarge)
    }

    payload.write_u16::<BigEndian>(ods.width)?;
    payload.write_u16::<BigEndian>(ods.height)?;
    payload.write_all(&ods.data)?;

    Ok(payload)
}

fn generate_mods(ods: &MiddleObjectDefinitionSegment) -> WriteResult<Vec<u8>> {

    // TODO: Validate data size (16-bit).

    let mut payload = vec![];

    payload.write_u16::<BigEndian>(ods.id)?;
    payload.write_u8(ods.version)?;
    payload.write_u8(0x00)?;
    payload.write_all(&ods.data)?;

    Ok(payload)
}

fn generate_fods(ods: &FinalObjectDefinitionSegment) -> WriteResult<Vec<u8>> {

    // TODO: Validate data size (16-bit).

    let mut payload = vec![];

    payload.write_u16::<BigEndian>(ods.id)?;
    payload.write_u8(ods.version)?;
    payload.write_u8(0x40)?;
    payload.write_all(&ods.data)?;

    Ok(payload)
}
