/*
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a
 * copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 William Swartzendruber
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use super::{
    DisplaySet,
    super::segment::{
        CompositionObject,
        EndSegment,
        FinalObjectDefinitionSegment,
        InitialObjectDefinitionSegment,
        MiddleObjectDefinitionSegment,
        PaletteDefinitionSegment,
        PaletteEntry,
        PresentationCompositionSegment,
        SingleObjectDefinitionSegment,
        WindowDefinition,
        WindowDefinitionSegment,
        WriteError as SegmentWriteError,
        WriteSegmentExt,
        Segment,
    },
};
use std::io::Write;
use thiserror::Error as ThisError;

const IODS_DATA_SIZE: usize = 65_508;
const MODS_DATA_SIZE: usize = 65_515;

pub type WriteResult<T> = Result<T, WriteError>;

#[derive(ThisError, Debug)]
pub enum WriteError {
    #[error("segment value error")]
    SegmentError {
        #[from]
        source: SegmentWriteError,
    },
    #[error("composition references unknown object ID")]
    CompositionReferencesUnknownObjectId,
    #[error("composition references unknown window ID")]
    CompositionReferencesUnknownWindowId,
    /// The [`Segment`] ([`ObjectDefinitionSegment`]) being written has a line with more than
    /// 16,383 pixels.
    #[error("object line too long")]
    ObjectLineTooLong,
}

pub trait WriteDisplaySetExt {
    fn write_display_set(&mut self, display_set: &DisplaySet) -> WriteResult<()>;
}

impl<T> WriteDisplaySetExt for T where
    T: Write,
{

    fn write_display_set(&mut self, display_set: &DisplaySet) -> WriteResult<()> {

        self.write_segment(&Segment::PresentationComposition(
            PresentationCompositionSegment {
                pts: display_set.pts,
                dts: display_set.dts,
                width: display_set.width,
                height: display_set.height,
                frame_rate: display_set.frame_rate,
                composition_number: display_set.composition.number,
                composition_state: display_set.composition.state,
                palette_update_id: display_set.palette_update_id,
                composition_objects: display_set.composition.objects.iter().map(|(cid, co)|
                    CompositionObject {
                        object_id: cid.object_id,
                        window_id: cid.window_id,
                        x: co.x,
                        y: co.y,
                        crop: co.crop.clone(),
                    }
                ).collect::<Vec<CompositionObject>>(),
            }
        ))?;

        if !display_set.windows.is_empty() {
            self.write_segment(&Segment::WindowDefinition(
                WindowDefinitionSegment {
                    pts: display_set.pts,
                    dts: display_set.dts,
                    windows: display_set.windows.iter().map(|(&window_id, window)|
                        WindowDefinition {
                            id: window_id,
                            x: window.x,
                            y: window.y,
                            width: window.width,
                            height: window.height,
                        }
                    ).collect::<Vec<WindowDefinition>>(),
                }
            ))?;
        }

        for (vid, palette) in display_set.palettes.iter() {
            self.write_segment(&Segment::PaletteDefinition(
                PaletteDefinitionSegment {
                    pts: display_set.pts,
                    dts: display_set.dts,
                    id: vid.id,
                    version: vid.version,
                    entries: palette.entries.iter().map(|(&id, entry)|
                        PaletteEntry {
                            id,
                            y: entry.y,
                            cr: entry.cr,
                            cb: entry.cb,
                            alpha: entry.alpha,
                        }
                    ).collect::<Vec<PaletteEntry>>(),
                }
            ))?;
        }

        for (vid, object) in display_set.objects.iter() {

            let data = rle_compress(&object.lines)?;
            let mut index = 0;
            let mut size = data.len();

            if size > IODS_DATA_SIZE {
                self.write_segment(&Segment::InitialObjectDefinition(
                    InitialObjectDefinitionSegment {
                        pts: display_set.pts,
                        dts: display_set.dts,
                        id: vid.id,
                        version: vid.version,
                        width: object.width,
                        height: object.height,
                        length: data.len() + 4,
                        data: Vec::from(&data[..IODS_DATA_SIZE]),
                    }
                ))?;
                index += IODS_DATA_SIZE;
                size -= IODS_DATA_SIZE;
                while size > MODS_DATA_SIZE {
                    self.write_segment(&Segment::MiddleObjectDefinition(
                        MiddleObjectDefinitionSegment {
                            pts: display_set.pts,
                            dts: display_set.dts,
                            id: vid.id,
                            version: vid.version,
                            data: Vec::from(&data[index..(index + MODS_DATA_SIZE)]),
                        }
                    ))?;
                    index += MODS_DATA_SIZE;
                    size -= MODS_DATA_SIZE;
                }
                self.write_segment(&Segment::FinalObjectDefinition(
                    FinalObjectDefinitionSegment {
                        pts: display_set.pts,
                        dts: display_set.dts,
                        id: vid.id,
                        version: vid.version,
                        data: Vec::from(&data[index..]),
                    }
                ))?;
            } else {
                self.write_segment(&Segment::SingleObjectDefinition(
                    SingleObjectDefinitionSegment {
                        pts: display_set.pts,
                        dts: display_set.dts,
                        id: vid.id,
                        version: vid.version,
                        width: object.width,
                        height: object.height,
                        data,
                    }
                ))?;
            }
        }

        self.write_segment(&Segment::End(
            EndSegment {
                pts: display_set.pts,
                dts: display_set.dts,
            }
        ))?;

        Ok(())
    }
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
            64 ..= 16_383 => {
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
            64 ..= 16_383 => {
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
