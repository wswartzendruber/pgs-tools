/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: OSL-3.0
 */

#[cfg(test)]
mod tests;

mod segmentread;
mod segmentwrite;

pub use segmentread::*;
pub use segmentwrite::*;

#[derive(Clone, Debug, Hash, PartialEq)]
pub enum Segment {
    PresentationComposition(PresentationCompositionSegment),
    WindowDefinition(WindowDefinitionSegment),
    PaletteDefinition(PaletteDefinitionSegment),
    ObjectDefinition(ObjectDefinitionSegment),
    End(EndSegment),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CompositionState {
    Normal,
    AcquisitionPoint,
    EpochStart,
}

impl Default for CompositionState {
    fn default() -> Self { Self::EpochStart }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Sequence {
    Single,
    First,
    Last,
}

impl Default for Sequence {
    fn default() -> Self { Self::Single }
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
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

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct CompositionObject {
    pub object_id: u16,
    pub window_id: u8,
    pub x: u16,
    pub y: u16,
    pub crop: Option<Crop>,
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct Crop {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct WindowDefinitionSegment {
    pub pts: u32,
    pub dts: u32,
    pub windows: Vec<WindowDefinition>,
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct WindowDefinition {
    pub id: u8,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct PaletteDefinitionSegment {
    pub pts: u32,
    pub dts: u32,
    pub id: u8,
    pub version: u8,
    pub entries: Vec<PaletteEntry>,
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct PaletteEntry {
    pub id: u8,
    pub y: u8,
    pub cr: u8,
    pub cb: u8,
    pub alpha: u8,
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct ObjectDefinitionSegment {
    pub pts: u32,
    pub dts: u32,
    pub id: u16,
    pub version: u8,
    pub sequence: Sequence,
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct EndSegment {
    pub pts: u32,
    pub dts: u32,
}
