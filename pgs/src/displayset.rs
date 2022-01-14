/*
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a
 * copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 William Swartzendruber
 *
 * SPDX-License-Identifier: MPL-2.0
 */

//! Operates on complete display sets (DS's).
//!
//! # Overview
//!
//! A display set (DS) is a collection of segments. Multiple DS's come together to form an
//! epoch.
//!
//! A single DS will typically perform one of the following functions for the epoch it resides
//! in:
//!
//! - Define and apply a new composition to the screen.
//! - Redefine an existing composition (allows the player to seek directly to this DS).
//! - Update a composition that is already on the screen.
//! - Remove a composition from the screen.
//!
//! In this way, an epoch can be seen as using multiple DS entites to perform the following
//! procedure:
//!
//! 1. Apply a composition to the screen.
//! 2. Update it (if desired).
//! 3. Remove it.
//!
//! As such, an epoch may simply be thought of as text that appears on the screen for a period
//! of time.

#[cfg(test)]
mod tests;

mod displaysetread;
mod displaysetwrite;

pub use displaysetread::*;
pub use displaysetwrite::*;

use std::collections::BTreeMap;
use super::segment::{Crop, CompositionState};

/// Represents a complete display set (DS) within an epoch.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct DisplaySet {
    /// The timestamp indicating when composition decoding should start. In practice, this is
    /// the time at which the composition is displayed.
    pub pts: u32,
    /// The timestamp indicating when the composition should be displayed.
    pub dts: u32,
    /// The width of the screen in pixels. This value should be consistent within a
    /// presentation.
    pub width: u16,
    /// The height of the screen in pixels. This value should be consistent within a
    /// presentation.
    pub height: u16,
    /// This value should be set to `0x10` but can otherwise be typically ignored.
    pub frame_rate: u8,
    /// If set, indicates the ID of a preceding palette to be updated within the epoch.
    pub palette_update_id: Option<u8>,
    /// The collection of windows referenced by this DS.
    pub windows: BTreeMap<u8, Window>,
    /// The collection of palettes referenced by this DS.
    pub palettes: BTreeMap<Vid<u8>, Palette>,
    /// The collection of objects referenced by this DS.
    pub objects: BTreeMap<Vid<u16>, Object>,
    /// Defines the composition of objects into windows.
    pub composition: Composition,
}

/// Represents a composition of objects into windows.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct Composition {
    /// Starting at zero, this increments each time graphics are updated within an epoch.
    pub number: u16,
    /// Defines the role of this DS within the larger epoch.
    pub state: CompositionState,
    /// A collection of [CompositionObject]s, each mapped according to its compound ID (object
    /// ID + window ID).
    pub objects: BTreeMap<Cid, CompositionObject>,
}

/// Defines a compound ID, combining an object and window identifier.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Cid {
    /// The object ID.
    pub object_id: u16,
    /// The window ID.
    pub window_id: u8,
}

/// Defines the location of an object (or a region of one) within a window.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct CompositionObject {
    /// The horizontal offset of the object’s top-left corner relative to the top-left corner of
    /// the screen. If the object is cropped, then this applies only to the visible area.
    pub x: u16,
    /// The vertical offset of the object’s top-left corner relative to the top-left corner of
    /// the screen. If the object is cropped, then this applies only to the visible area.
    pub y: u16,
    /// If set, defines the visible area of the object. Otherwise, the entire object is shown.
    pub crop: Option<Crop>,
}

/// Defines a window within a display set.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct Window {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Defines a palette within a display set.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct Palette {
    /// The entries within this palette, each mapped according to its ID.
    pub entries: BTreeMap<u8, PaletteEntry>
}

/// Defines a palette entry within a palette set.
///
/// The role of a palette entry is to define or update exact pixel color, as later referenced by
/// any objects also defined within an epoch.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct PaletteEntry {
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

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct Object {
    pub width: u16,
    pub height: u16,
    pub lines: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Vid<T> {
    pub id: T,
    pub version: u8,
}
