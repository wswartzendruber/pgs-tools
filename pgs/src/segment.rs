/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: OSL-3.0
 */

mod segmentread;
mod segmentwrite;

pub use segmentread::*;
pub use segmentwrite::*;

pub enum Segment {
    PresentationComposition(PresentationCompositionSegment),
    WindowDefinition(WindowDefinitionSegment),
    PaletteDefinition(PaletteDefinitionSegment),
    ObjectDefinition(ObjectDefinitionSegment),
    End(EndSegment),
}

pub enum CompositionState {
    Normal,
    AcquisitionPoint,
    EpochStart,
}

pub enum ObjectSequence {
    Last,
    First,
    Both,
}

pub struct PresentationCompositionSegment {
    pub pts: u32,
    pub dts: u32,
    pub width: u16,
    pub height: u16,
    pub frame_rate: u8,
    pub composition_number: u16,
    pub composition_state: CompositionState,
    pub palette_update_id: Option<u8>,
    pub composition_objects: Vec<CompositionObject>,
}

pub struct CompositionObject {
    pub object_id: u16,
    pub window_id: u8,
    pub x: u16,
    pub y: u16,
    pub crop: Option<Crop>,
}

pub struct Crop {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

pub struct WindowDefinitionSegment {
    pub pts: u32,
    pub dts: u32,
    pub windows: Vec<WindowDefinition>,
}

pub struct WindowDefinition {
    pub id: u8,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

pub struct PaletteDefinitionSegment {
    pub pts: u32,
    pub dts: u32,
    pub id: u8,
    pub version: u8,
    pub entries: Vec<PaletteEntry>,
}

pub struct PaletteEntry {
    pub id: u8,
    pub y: u8,
    pub cr: u8,
    pub cb: u8,
    pub alpha: u8,
}

pub struct ObjectDefinitionSegment {
    pub pts: u32,
    pub dts: u32,
    pub id: u16,
    pub version: u8,
    pub sequence: Option<ObjectSequence>,
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

pub struct EndSegment {
    pub pts: u32,
    pub dts: u32,
}
