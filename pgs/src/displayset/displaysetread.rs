/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: OSL-3.0
 */

use super::{
    Cid,
    Composition,
    CompositionObject,
    DisplaySet,
    Object,
    Palette,
    PaletteEntry,
    Vid,
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

pub type ReadResult<T> = Result<T, ReadError>;

#[derive(ThisError, Debug)]
pub enum ReadError {
    #[error("segment value error")]
    SegmentError {
        #[from]
        source: SegmentReadError,
    },
    #[error("first segment is not a presentation composition segment")]
    MissingPresentationCompositionSegment,
    #[error("PTS is not consistent with presentation composition segment")]
    InconsistentPts,
    #[error("DTS is not consistent with presentation composition segment")]
    InconsistentDts,
    #[error("unexpected presentation composition segment within display set")]
    UnexpectedPresentationCompositionSegment,
    #[error("duplicate window ID detected")]
    DuplicateWindowId,
    #[error("duplicate palette ID and version detected")]
    DuplicatePaletteVid,
    #[error("duplicate object ID and version detected")]
    DuplicateObjectVid,
    #[error("composition references unknown object ID")]
    CompositionReferencesUnknownObjectId,
    #[error("composition references unknown window ID")]
    CompositionReferencesUnknownWindowId,
    #[error("palette update references unknown palette ID")]
    PaletteUpdateReferencesUnknownPaletteId,
}

pub trait ReadDisplaySetExt {
    fn read_display_set(&mut self) -> ReadResult<DisplaySet>;
}

impl<T> ReadDisplaySetExt for T where
    T: Read,
{

    fn read_display_set(&mut self) -> ReadResult<DisplaySet> {

        let mut windows = BTreeMap::<u8, Window>::new();
        let mut palettes = BTreeMap::<Vid<u8>, Palette>::new();
        let mut objects = BTreeMap::<Vid<u16>, Object>::new();
        let mut composition_objects = BTreeMap::<Cid, CompositionObject>::new();
        let first_seg = self.read_segment()?;
        let pcs = match first_seg {
            Segment::PresentationComposition(pcs) => pcs,
            _ => return Err(ReadError::MissingPresentationCompositionSegment),
        };
        let pts = pcs.pts;
        let dts = pcs.dts;

        loop {

            let segment = self.read_segment()?;

            match segment {
                Segment::PresentationComposition(_) => {
                    return Err(ReadError::UnexpectedPresentationCompositionSegment)
                }
                Segment::WindowDefinition(wds) => {
                    if wds.pts != pts {
                        return Err(ReadError::InconsistentPts)
                    }
                    if wds.dts != dts {
                        return Err(ReadError::InconsistentDts)
                    }
                    for wd in wds.windows.iter() {
                        if windows.contains_key(&wd.id) {
                            return Err(ReadError::DuplicateWindowId)
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
                        return Err(ReadError::InconsistentPts)
                    }
                    if pds.dts != dts {
                        return Err(ReadError::InconsistentDts)
                    }
                    let vid = Vid {
                        id: pds.id,
                        version: pds.version,
                    };
                    if palettes.contains_key(&vid) {
                        return Err(ReadError::DuplicatePaletteVid)
                    }
                    palettes.insert(
                        vid,
                        Palette {
                            entries: pds.entries.iter().map(|pe|
                                (pe.id, PaletteEntry {
                                    y: pe.y,
                                    cr: pe.cr,
                                    cb: pe.cb,
                                    alpha: pe.alpha,
                                })
                            ).collect::<BTreeMap<u8, PaletteEntry>>()
                        },
                    );
                }
                Segment::ObjectDefinition(ods) => {
                    if ods.pts != pts {
                        return Err(ReadError::InconsistentPts)
                    }
                    if ods.dts != dts {
                        return Err(ReadError::InconsistentDts)
                    }
                    let vid = Vid {
                        id: ods.id,
                        version: ods.version,
                    };
                    if objects.contains_key(&vid) {
                        return Err(ReadError::DuplicateObjectVid)
                    }
                    objects.insert(
                        vid,
                        Object {
                            width: ods.width,
                            height: ods.height,
                            sequence: ods.sequence,
                            lines: ods.lines,
                        },
                    );
                }
                Segment::End(es) => {
                    if es.pts != pts {
                        return Err(ReadError::InconsistentPts)
                    }
                    if es.dts != dts {
                        return Err(ReadError::InconsistentDts)
                    }
                    break
                }
            }
        }

        for co in pcs.composition_objects.iter() {
            if !objects.keys().any(|vid| vid.id == co.object_id) {
                return Err(ReadError::CompositionReferencesUnknownObjectId)
            }
            if !windows.contains_key(&co.window_id) {
                return Err(ReadError::CompositionReferencesUnknownWindowId)
            }
            composition_objects.insert(
                Cid {
                    object_id: co.object_id,
                    window_id: co.window_id,
                },
                CompositionObject {
                    x: co.x,
                    y: co.y,
                    crop: co.crop.clone(),
                },
            );
        }

        let composition = Composition {
            number: pcs.composition_number,
            state: pcs.composition_state,
            objects: composition_objects,
        };

        match pcs.palette_update_id {
            Some(palette_update_id) => {
                if !palettes.keys().any(|vid| vid.id == palette_update_id) {
                    return Err(ReadError::PaletteUpdateReferencesUnknownPaletteId)
                }
            }
            None => {
            }
        }

        Ok(
            DisplaySet {
                pts,
                dts,
                width: pcs.width,
                height: pcs.height,
                frame_rate: pcs.frame_rate,
                palette_update_id: pcs.palette_update_id,
                windows,
                palettes,
                objects,
                composition,
            }
        )
    }
}
