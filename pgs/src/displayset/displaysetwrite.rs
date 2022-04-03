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
    /// The [`Segment`] ([`ObjectDefinitionSegment`]) being written has a line with more than
    /// 16,383 pixels.
    #[error("object line too long")]
    ObjectLineTooLong,
}

pub trait WriteDisplaySetExt {
    fn write_display_set(&mut self, display_set: DisplaySet) -> WriteResult<()>;
}

impl<T> WriteDisplaySetExt for T where
    T: Write,
{

    fn write_display_set(&mut self, display_set: DisplaySet) -> WriteResult<()> {

        let segments = display_set.into_segments()?;

        for segment in segments.into_iter() {
            self.write_segment(&segment)?;
        }

        Ok(())
    }
}

impl DisplaySet {

    fn into_segments(self) -> WriteResult<Vec<Segment>> {

        let mut segments = Vec::<Segment>::new();

        segments.push(Segment::PresentationComposition(
            PresentationCompositionSegment {
                pts: self.pts,
                dts: self.dts,
                width: self.width,
                height: self.height,
                frame_rate: self.frame_rate,
                composition_number: self.composition.number,
                composition_state: self.composition.state,
                palette_update_id: self.palette_update_id,
                composition_objects: self.composition.objects.iter().map(|(cid, co)|
                    CompositionObject {
                        object_id: cid.object_id,
                        window_id: cid.window_id,
                        x: co.x,
                        y: co.y,
                        forced: co.forced,
                        crop: co.crop.clone(),
                    }
                ).collect::<Vec<CompositionObject>>(),
            }
        ));

        if !self.windows.is_empty() {
            segments.push(Segment::WindowDefinition(
                WindowDefinitionSegment {
                    pts: self.pts,
                    dts: self.dts,
                    windows: self.windows.iter().map(|(&window_id, window)|
                        WindowDefinition {
                            id: window_id,
                            x: window.x,
                            y: window.y,
                            width: window.width,
                            height: window.height,
                        }
                    ).collect::<Vec<WindowDefinition>>(),
                }
            ));
        }

        for (vid, palette) in self.palettes.iter() {
            segments.push(Segment::PaletteDefinition(
                PaletteDefinitionSegment {
                    pts: self.pts,
                    dts: self.dts,
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
            ));
        }

        for (vid, object) in self.objects.iter() {

            let data = rle_compress(&object.lines)?;
            let mut index = 0;
            let mut size = data.len();

            if size > IODS_DATA_SIZE {
                segments.push(Segment::InitialObjectDefinition(
                    InitialObjectDefinitionSegment {
                        pts: self.pts,
                        dts: self.dts,
                        id: vid.id,
                        version: vid.version,
                        width: object.width,
                        height: object.height,
                        length: data.len() + 4,
                        data: Vec::from(&data[..IODS_DATA_SIZE]),
                    }
                ));
                index += IODS_DATA_SIZE;
                size -= IODS_DATA_SIZE;
                while size > MODS_DATA_SIZE {
                    segments.push(Segment::MiddleObjectDefinition(
                        MiddleObjectDefinitionSegment {
                            pts: self.pts,
                            dts: self.dts,
                            id: vid.id,
                            version: vid.version,
                            data: Vec::from(&data[index..(index + MODS_DATA_SIZE)]),
                        }
                    ));
                    index += MODS_DATA_SIZE;
                    size -= MODS_DATA_SIZE;
                }
                segments.push(Segment::FinalObjectDefinition(
                    FinalObjectDefinitionSegment {
                        pts: self.pts,
                        dts: self.dts,
                        id: vid.id,
                        version: vid.version,
                        data: Vec::from(&data[index..]),
                    }
                ));
            } else {
                segments.push(Segment::SingleObjectDefinition(
                    SingleObjectDefinitionSegment {
                        pts: self.pts,
                        dts: self.dts,
                        id: vid.id,
                        version: vid.version,
                        width: object.width,
                        height: object.height,
                        data,
                    }
                ));
            }
        }

        segments.push(Segment::End(
            EndSegment {
                pts: self.pts,
                dts: self.dts,
            }
        ));

        Ok(segments)
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
