/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: OSL-3.0
 */

use super::{
    DisplaySet,
    Palette,
    PaletteEntry,
    Window,
    super::segment::{
        ReadError as SegmentReadError,
        ReadSegmentExt,
        Segment,
    },
};
use std::{
    collections::BTreeMap,
    io::Read,
};
use thiserror::Error as ThisError;

pub type DisplaySetReadResult<T> = Result<T, DisplaySetReadError>;

#[derive(ThisError, Debug)]
pub enum DisplaySetReadError {
    #[error("segment value error")]
    IoError {
        #[from]
        source: SegmentReadError,
    },
    #[error("first segment is not a presentation composition segment")]
    ExpectedPresCompSeg,
    #[error("PTS is not consistent with presentation composition segment")]
    InconsistentPts,
    #[error("DTS is not consistent with presentation composition segment")]
    InconsistentDts,
    #[error("presentation composition segment not expected within display set")]
    UnexpectedPresCompSeg,
    #[error("duplicate window ID detected")]
    DuplicateWindowId,
    #[error("duplicate palette definition segment detected")]
    DuplicatePalDefSeg,
}

pub trait ReadDisplaySetExt {
    fn read_display_set(&mut self) -> DisplaySetReadResult<DisplaySet>;
}

impl<T: Read> ReadDisplaySetExt for T {

    fn read_display_set(&mut self) -> DisplaySetReadResult<DisplaySet> {

        let mut windows = BTreeMap::<u8, Window>::new();
        let mut palettes = BTreeMap::<u8, Palette>::new();
        let first_seg = self.read_segment()?;
        let pcs = match first_seg {
            Segment::PresentationComposition(pcs) => pcs,
            _ => return Err(DisplaySetReadError::ExpectedPresCompSeg),
        };
        let pts = pcs.pts;
        let dts = pcs.dts;

        loop {

            let segment = self.read_segment()?;

            match segment {
                Segment::PresentationComposition(_) => {
                    return Err(DisplaySetReadError::UnexpectedPresCompSeg)
                }
                Segment::WindowDefinition(wds) => {
                    if wds.pts != pts {
                        return Err(DisplaySetReadError::InconsistentPts)
                    }
                    if wds.dts != dts {
                        return Err(DisplaySetReadError::InconsistentDts)
                    }
                    for wd in wds.windows.iter() {
                        if windows.contains_key(&wd.id) {
                            return Err(DisplaySetReadError::DuplicateWindowId)
                        }
                        windows.insert(
                            wd.id,
                            Window {
                                x: wd.x,
                                y: wd.y,
                                width: wd.width,
                                height: wd.height,
                            },
                        );
                    }
                }
                Segment::PaletteDefinition(pds) => {
                    if pds.pts != pts {
                        return Err(DisplaySetReadError::InconsistentPts)
                    }
                    if pds.dts != dts {
                        return Err(DisplaySetReadError::InconsistentDts)
                    }
                    palettes.insert(
                        pds.id,
                        Palette {
                            version: pds.version,
                            entries: pds.entries.iter().map(|pe| (pe.id,
                                PaletteEntry {
                                    y: pe.y,
                                    cr: pe.cr,
                                    cb: pe.cb,
                                    alpha: pe.alpha,
                                }
                            )).collect::<BTreeMap<u8, PaletteEntry>>()
                        },
                    );
                }
                Segment::ObjectDefinition(ods) => {
                    if ods.pts != pts {
                        return Err(DisplaySetReadError::InconsistentPts)
                    }
                    if ods.dts != dts {
                        return Err(DisplaySetReadError::InconsistentDts)
                    }
                    // TODO
                }
                Segment::End(es) => {
                    if es.pts != pts {
                        return Err(DisplaySetReadError::InconsistentPts)
                    }
                    if es.dts != dts {
                        return Err(DisplaySetReadError::InconsistentDts)
                    }
                    break
                }
            }
        }

        Ok(
            DisplaySet {
                pts,
                dts,
                windows,
                palettes,
            }
        )
    }
}
