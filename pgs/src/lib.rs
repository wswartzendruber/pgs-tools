/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: OSL-3.0
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
//! a given time. It is also possible to display only a specific rectangular area of an object
//! inside of a window instead of the entire object.
//!
//! ## The Epoch
//!
//! The cycle for displaying a caption is called an epoch. Each epoch consists of multiple
//! display sets. The first diplay set defines the windows and objects to be shown. Subsequent
//! display sets may update the objects being shown, but the final display set ultimately
//! removes the objects from the screen area.
//!
//! ## Display Sets
//!
//! A display set is composed of multiple segments. There are several types and they do things
//! like mark the beginning of a display set, define windows, define objects, define colors, and
//! mark the end of a display set.
//!
//! ## Segments
//!
//! A segment is a structure of well-defined properties. Some of these properties are optional
//! and may not apply to a given case.
//!
//! # Using this Crate
//!
//! This crate supports two separate levels of PGS abstraction. That is, a user may choose to
//! operate on either complete display sets or on individual segments, depending on what they
//! are trying to achieve. The segment API, for example, is more suited towards writing
//! diagnostics tooling and other low level components. The display set API, on the other hand,
//! is more suited towards writing tooling that modifies stream properties, like window
//! positioning and object colors.

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
