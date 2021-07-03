/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: OSL-3.0
 */

// #[cfg(test)]
// mod tests;

mod displaysetread;
//mod displaysetwrite;

pub use displaysetread::*;
//pub use displaysetwrite::*;

use std::collections::BTreeMap;
use super::segment::Crop;

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct DisplaySet {
    pub pts: u32,
    pub dts: u32,
    pub compositions: BTreeMap<Cid, Composition>,
    pub windows: BTreeMap<u8, Window>,
    pub palettes: BTreeMap<Vid<u8>, Palette>,
    pub objects: BTreeMap<Vid<u16>, Object>,
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Cid {
    pub object_id: u16,
    pub window_id: u8,
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct Composition {
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
    pub lines: Vec<Vec<u8>>,
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Vid<T> {
    pub id: T,
    pub version: u8,
}
