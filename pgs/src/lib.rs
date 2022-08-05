/*
 * Copyright 2021 William Swartzendruber
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a
 * copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

//! # Introduction
//!
//! This crate facilitates the encoding and decoding of Presentation Graphics Stream subtitles,
//! which are commonly used in Blu-ray movie discs. These are often times referred to as SUP
//! subtitles.
//!
//! Unfortunately, PGS has no publicly available documentation. Therefore, the behavior of this
//! crate has been defined using a hierarchy of sources. For any aspect of behavior,
//! [U.S. Patent US 20090185789A1](https://patents.google.com/patent/US20090185789/da) is
//! consulted for initial understanding. This is used as-is unless that information happens to
//! conflict with [a helpful blog post about PGS](http://blog.thescorpius.com/index.php/2017/07/15/presentation-graphic-stream-sup-files-bluray-subtitle-format/),
//! in which case that information is used instead. Should either or both of these sources ever
//! conflict with observations from commercial Blu-ray discs, then the correct behavior is
//! reverse-engineered based on what is encountered on those discs.
//!
//! # PGS Overview
//!
//! PGS works by defining a screen area for all captions to use. Within this area, window
//! regions are defined. Objects are then placed within each window. Any number of windows may
//! be in the screen area at a given time, but each window may have no more than two objects at
//! a given time. It is also possible to show only a specific rectangular area of an object
//! inside of a window instead of the entire object.
//!
//! The process of rendering objects to the screen is known as composition.
//!
//! ## The Epoch
//!
//! Conceptually, an epoch displays one or more captions to specific areas of the screen. As
//! such, each one defines fixed window positions, but may swap out objects (or portions of
//! objects) being shown in those windows as the epoch progresses.
//!
//! An epoch is composed of multiple display sets (DS).
//!
//! ## Display Sets
//!
//! A display set (DS) is a set of instructions for how to compose an epoch throughout that
//! epoch's lifetime. Each DS within an epoch has one of three roles:
//!
//! 1. Initiate a new epoch (define windows, objects, and palettes)
//! 2. Refresh the player regarding the current epoch (should the player seek past the first DS)
//! 3. Make changes to the current epoch, including tearing it down
//!
//! A DS is composed of multiple segments.
//!
//! ## Segments
//!
//! A segment defines a specific set of properties for a display set (DS). There are five types
//! of segments, according to the properties each one defines. One defines properties for the
//! entire DS, another defines windows, a third defines objects, a fourth defines palettes, and
//! a fifth is used to signal the end of the current DS.
//!
//! Segments are separated because each type is not always needed. A DS that only updates the
//! color palettes of existing objects on the screen, for example, does not need to define new
//! objects or windows.
//!
//! # Using this Crate
//!
//! This crate supports two separate levels of PGS abstraction. That is, a user may choose to
//! operate on either complete display sets (DS) or on individual segments, depending on what
//! they are trying to achieve. The segment API, for example, is more suited towards writing
//! diagnostics tooling and other low level components. The display set API, on the other hand,
//! is more suited towards writing tooling that modifies stream properties, like window
//! positions and object colors.

pub mod displayset;
pub mod segment;

pub fn ts_to_timestamp(ts: u32) -> String {

    let mut ms = ts / 90;
    let h = ms / 3_600_000;
    ms -= h * 3_600_000;
    let m = ms / 60_000;
    ms -= m * 60_000;
    let s = ms / 1_000;
    ms -= s * 1_000;

    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms)
}
