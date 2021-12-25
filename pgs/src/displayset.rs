/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: OSL-3.0
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
use super::segment::{Crop, CompositionState, Sequence};

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

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct Composition {
    pub number: u16,
    pub state: CompositionState,
    pub objects: BTreeMap<Cid, CompositionObject>,
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Cid {
    pub object_id: u16,
    pub window_id: u8,
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct CompositionObject {
    pub x: u16,
    pub y: u16,
    pub crop: Option<Crop>,
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct Window {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct Palette {
    pub entries: BTreeMap<u8, PaletteEntry>
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct PaletteEntry {
    pub y: u8,
    pub cr: u8,
    pub cb: u8,
    pub alpha: u8,
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct Object {
    pub width: u16,
    pub height: u16,
    pub sequence: Sequence,
    pub lines: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Vid<T> {
    pub id: T,
    pub version: u8,
}
