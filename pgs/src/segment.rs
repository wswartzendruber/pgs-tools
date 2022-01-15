/*
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a
 * copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 William Swartzendruber
 *
 * SPDX-License-Identifier: MPL-2.0
 */

//! Operates on individual segments.
//!
//! # Overview
//!
//! A segment is the most fundamental data structure within a PGS bitstream. Multiple segments
//! come together in a well-defined manner to form a display set (DS).
//!
//! There are five types that typically appear in this order:
//!
//! 1. Presentation Composition Segment (PCS)
//! 2. Window Definition Segment (WDS)
//! 3. Palette Definition Segment (PDS)
//! 4. Object Definition Segment (ODS)
//! 5. End Segment (ES)
//!
//! Something the fives types have in common is that each one defines both PTS and DTS
//! timestamps. They are typically identical within a given DS.
//!
//! ## Presentation Composition Segment (PCS)
//!
//! A PCS signals the start of a new display set (DS). It also defines properties such as the
//! role of the DS within the larger epoch, the screen resolution, and initial mappings of
//! objects to windows.
//!
//! See: [PresentationCompositionSegment]
//!
//! ## Window Definition Segment (WDS)
//!
//! A WDS defines the areas of the screen that will be used to show objects during the larger
//! epoch. As a single WDS can define multiple windows, each DS should only have one.
//!
//! See: [WindowDefinitionSegment]
//!
//! ## Palette Definition Segment (PDS)
//!
//! A PDS contains a list of YCbCrA values with each one having a unique ID. A single DS can
//! have multiple PDS segments.
//!
//! See: [PaletteDefinitionSegment]
//!
//! ## Object Definition Segment (ODS)
//!
//! An ODS defines a sequence of pixels with each pixel consisting of a single ID. These IDs map
//! back to the pixel values encountered in earlier PDS segments.
//!
//! See: [ObjectDefinitionSegment]
//!
//! ## End Segment (ES)
//!
//! An ES signals that the current DS has come to an end.
//!
//! See: [EndSegment]

#[cfg(test)]
mod tests;

mod segmentread;
mod segmentwrite;

pub use segmentread::*;
pub use segmentwrite::*;

/// Represents a PGS segment.
#[derive(Clone, Debug, Hash, PartialEq)]
pub enum Segment {
    /// Represents a Presentation Composition Segment (PCS).
    PresentationComposition(PresentationCompositionSegment),
    /// Represents a Window Definition Segment (WDS).
    WindowDefinition(WindowDefinitionSegment),
    /// Represents a Palette Definition Segment (PDS).
    PaletteDefinition(PaletteDefinitionSegment),
    /// Represents a complete Object Definition Segment (ODS).
    SingleObjectDefinition(SingleObjectDefinitionSegment),
    /// Represents the initial portion of an Object Definition Segment (ODS).
    InitialObjectDefinition(InitialObjectDefinitionSegment),
    /// Represents a middle portion of an Object Definition Segment (ODS).
    MiddleObjectDefinition(MiddleObjectDefinitionSegment),
    /// Represents the final portion of an Object Definition Segment (ODS).
    FinalObjectDefinition(FinalObjectDefinitionSegment),
    /// Represents an End Segment (ES).
    End(EndSegment),
}

/// Defines the role of a PCS (and thereby the associated DS) within an epoch.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CompositionState {
    /// Indicates that the associated PCS (and the DS it belongs to) defines the start of a new
    /// epoch. As such, the associated DS should contain all other segments necessary to render
    /// a composition onto the screen.
    EpochStart,
    /// Similar to `EpochStart`, except used to refresh the screen with the current composition.
    /// That is, the associated DS should redefine the same windows, objects, and palettes as
    /// the `EpochStart` DS. This allows, for example, a player to seek past an `EpochStart` and
    /// land in the middle of an epoch, while still being able to show the relevant composition
    /// once the `AcquisitionPoint` is encountered. While it is technically possible to use this
    /// to alter the composition from what the `EpochStart` DS has defined, this practice is
    /// discouraged.
    AcquisitionPoint,
    /// This updates the composition that is on the screen. This is typically used to clear the
    /// current composition from the screen by defining a PCS with no composition objects,
    /// thereby effectively closing out the current epoch. But other things like palette updates
    /// and object substitution within a window can also be done. As an epoch is supposed to
    /// compose to fixed areas of the screen, redefining windows here is discouraged.
    Normal,
}

impl Default for CompositionState {
    fn default() -> Self { Self::EpochStart }
}

/// Defines a Presentation Composition Segment (PCS).
///
/// A PCS marks the beginning of a display set (DS).
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct PresentationCompositionSegment {
    /// The timestamp indicating when composition decoding should start. In practice, this is
    /// the time at which the composition is displayed. All segments within a DS typically have
    /// identical values here.
    pub pts: u32,
    /// The timestamp indicating when the composition should be displayed. In practice, this
    /// value is always zero.
    pub dts: u32,
    /// The width of the screen in pixels. This value should be consistent within a
    /// presentation.
    pub width: u16,
    /// The height of the screen in pixels. This value should be consistent within a
    /// presentation.
    pub height: u16,
    /// This value should be set to `0x10` but can otherwise be typically ignored.
    pub frame_rate: u8,
    /// Starting at zero, this increments each time graphics are updated within an epoch.
    pub composition_number: u16,
    /// Defines the role of the current DS within the larger epoch.
    pub composition_state: CompositionState,
    /// If set, indicates the ID of a preceding palette to be updated within the epoch.
    pub palette_update_id: Option<u8>,
    /// Maps an epoch's objects (or areas within them) to its windows.
    pub composition_objects: Vec<CompositionObject>,
}

/// Defines a mapping between an object (or an area of one) and a window within an epoch.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct CompositionObject {
    /// The ID of the object within the epoch.
    pub object_id: u16,
    /// The ID of the window within the epoch.
    pub window_id: u8,
    /// The horizontal offset of the object's top-left corner relative to the top-left corner of
    /// the screen. If the object is cropped, then this applies only to the visible area.
    pub x: u16,
    /// The vertical offset of the object's top-left corner relative to the top-left corner of
    /// the screen. If the object is cropped, then this applies only to the visible area.
    pub y: u16,
    /// If set, defines the visible area of the object. Otherwise, the entire object is shown.
    pub crop: Option<Crop>,
}

/// Defines the specific area within an object to be shown.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct Crop {
    /// The horizontal offset of the area's top-left corner relative to the top-left corner of
    /// the object itself.
    pub x: u16,
    /// The vertical offset of the area's top-left corner relative to the top-left corner of the
    /// object itself.
    pub y: u16,
    /// The width of the area.
    pub width: u16,
    /// The height of the area.
    pub height: u16,
}

/// Defines a Window Definition Segment (WDS).
///
/// A WDS lists window regions that are to be used within an epoch. Each DS that has a WDS
/// should only have one.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct WindowDefinitionSegment {
    /// The timestamp indicating when composition decoding should start. In practice, this is
    /// the time at which the composition is displayed. All segments within a DS typically have
    /// identical values here.
    pub pts: u32,
    /// The timestamp indicating when the composition should be displayed. In practice, this
    /// value is always zero.
    pub dts: u32,
    /// Defines the window regions within the screen for this epoch.
    pub windows: Vec<WindowDefinition>,
}

/// Defines a window within the screen.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct WindowDefinition {
    /// The ID of this window within the epoch.
    pub id: u8,
    /// The horizontal offset of the window's top-left corner relative to the top-left corner of
    /// the screen.
    pub x: u16,
    /// The vertical offset of the window's top-left corner relative to the top-left corner of
    /// the screen.
    pub y: u16,
    /// The width of the window.
    pub width: u16,
    /// The height of the window.
    pub height: u16,
}

/// Defines a set of palette entries within an epoch.
///
/// Palette entries can be broken apart into sets so that they can be modified as a group within
/// an epoch. This can be used, for example, to provide a fade-out effect by continuously
/// updating the palette entries referenced by an object currently on the screen.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct PaletteDefinitionSegment {
    /// The timestamp indicating when composition decoding should start. In practice, this is
    /// the time at which the composition is displayed. All segments within a DS typically have
    /// identical values here.
    pub pts: u32,
    /// The timestamp indicating when the composition should be displayed. In practice, this
    /// value is always zero.
    pub dts: u32,
    /// The ID of this pallete set within the epoch.
    pub id: u8,
    /// The version increment of this palette set.
    pub version: u8,
    /// Defines the individual palette entries in this set.
    pub entries: Vec<PaletteEntry>,
}

/// Defines a palette entry within a palette set.
///
/// The role of a palette entry is to define or update exact pixel color, as later referenced by
/// any objects also defined within an epoch.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct PaletteEntry {
    /// The ID of this palette entry, which should be unique within an epoch.
    pub id: u8,
    /// The range-limited, gamma-corrected luminosity value of this entry. Black is represented
    /// by a value of `16` while white is represented by a value of `235`. For standard Blu-ray
    /// discs, the BT.709 gamma function is typically used. However, 4K UltraHD discs seem to
    /// use the ST.2084 gamma function instead.
    pub y: u8,
    /// The vertical position of this entry on the YC<sub>b</sub>C<sub>r</sub> color plane,
    /// starting from the bottom and going up.
    pub cr: u8,
    /// The horizontal position of this entry on the YC<sub>b</sub>C<sub>r</sub> color plane,
    /// starting from the left and going to the right.
    pub cb: u8,
    /// The alpha value (transparency ratio) of this entry.
    pub alpha: u8,
}

/// Defines a complete object within an epoch.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct SingleObjectDefinitionSegment {
    /// The timestamp indicating when composition decoding should start. In practice, this is
    /// the time at which the composition is displayed. All segments within a DS typically have
    /// identical values here.
    pub pts: u32,
    /// The timestamp indicating when the composition should be displayed. In practice, this
    /// value is always zero.
    pub dts: u32,
    /// The ID of this object, which may be redefined within an epoch.
    pub id: u16,
    /// The version increment of this object.
    pub version: u8,
    /// The width of this object.
    pub width: u16,
    /// The height of this object.
    pub height: u16,
    /// The RLE-compressed data for this object.
    pub data: Vec<u8>,
}

/// Defines the initial portion of an object within an epoch.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct InitialObjectDefinitionSegment {
    /// The timestamp indicating when composition decoding should start. In practice, this is
    /// the time at which the composition is displayed. All segments within a DS typically have
    /// identical values here.
    pub pts: u32,
    /// The timestamp indicating when the composition should be displayed. In practice, this
    /// value is always zero.
    pub dts: u32,
    /// The ID of this object, which may be redefined within an epoch.
    pub id: u16,
    /// The version increment of this object.
    pub version: u8,
    /// The declared length of this object's data buffer, including all follow-on portions.
    pub length: usize,
    /// The width of this object.
    pub width: u16,
    /// The height of this object.
    pub height: u16,
    /// The RLE-compressed data for this portion of the completed object.
    pub data: Vec<u8>,
}

/// Defines a middle portion of an object within an epoch.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct MiddleObjectDefinitionSegment {
    /// The timestamp indicating when composition decoding should start. In practice, this is
    /// the time at which the composition is displayed. All segments within a DS typically have
    /// identical values here.
    pub pts: u32,
    /// The timestamp indicating when the composition should be displayed. In practice, this
    /// value is always zero.
    pub dts: u32,
    /// The ID of this object, which may be redefined within an epoch.
    pub id: u16,
    /// The version increment of this object.
    pub version: u8,
    /// The RLE-compressed data for this portion of the completed object.
    pub data: Vec<u8>,
}

/// Defines the final portion of an object within an epoch.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct FinalObjectDefinitionSegment {
    /// The timestamp indicating when composition decoding should start. In practice, this is
    /// the time at which the composition is displayed. All segments within a DS typically have
    /// identical values here.
    pub pts: u32,
    /// The timestamp indicating when the composition should be displayed. In practice, this
    /// value is always zero.
    pub dts: u32,
    /// The ID of this object, which may be redefined within an epoch.
    pub id: u16,
    /// The version increment of this object.
    pub version: u8,
    /// The RLE-compressed data for this portion of the completed object.
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
/// Defines the end of a display set (DS).
pub struct EndSegment {
    /// The timestamp indicating when composition decoding should start. In practice, this is
    /// the time at which the composition is displayed. All segments within a DS typically have
    /// identical values here.
    pub pts: u32,
    /// The timestamp indicating when the composition should be displayed. In practice, this
    /// value is always zero.
    pub dts: u32,
}
