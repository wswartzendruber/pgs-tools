/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: OSL-3.0
 */

use super::{
    DisplaySet,
    super::segment::{
        CompositionObject,
        EndSegment,
        ObjectDefinitionSegment,
        PaletteDefinitionSegment,
        PaletteEntry,
        PresentationCompositionSegment,
        WindowDefinition,
        WindowDefinitionSegment,
        WriteError as SegmentWriteError,
        WriteSegmentExt,
        Segment,
    },
};
use std::io::Write;
use thiserror::Error as ThisError;

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
}

pub trait WriteDisplaySetExt {
    fn write_display_set(&mut self, display_set: &DisplaySet) -> WriteResult<()>;
}

impl<T: Write> WriteDisplaySetExt for T {

    fn write_display_set(&mut self, display_set: &DisplaySet) -> WriteResult<()> {

        let pcs = PresentationCompositionSegment {
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
        };
        let wds = WindowDefinitionSegment {
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
        };
        let pdss = display_set.palettes.iter().map(|(vid, palette)|
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
        ).collect::<Vec<PaletteDefinitionSegment>>();
        let odss = display_set.objects.iter().map(|(vid, object)|
            ObjectDefinitionSegment {
                pts: display_set.pts,
                dts: display_set.dts,
                id: vid.id,
                version: vid.version,
                sequence: object.sequence,
                width: object.width,
                height: object.height,
                lines: object.lines.clone(),
            }
        ).collect::<Vec<ObjectDefinitionSegment>>();

        self.write_segment(&Segment::PresentationComposition(pcs))?;
        self.write_segment(&Segment::WindowDefinition(wds))?;
        for pds in pdss.iter() {
            self.write_segment(&Segment::PaletteDefinition(pds.clone()))?;
        }
        for ods in odss.iter() {
            self.write_segment(&Segment::ObjectDefinition(ods.clone()))?;
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
