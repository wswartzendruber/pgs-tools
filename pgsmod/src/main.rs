/*
 * Copyright 2021 William Swartzendruber
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a
 * copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

#[cfg(test)]
mod tests;

mod rgb;

use pgs::{
    displayset::{
        Object,
        ReadDisplaySetExt,
        ReadError as DisplaySetReadError,
        WriteDisplaySetExt,
    },
    segment::{
        CompositionState,
        ReadError as SegmentReadError,
    },
};
use rgb::{rgb_pixel, ycbcr_pixel, YcbcrPixel};
use std::{
    collections::HashMap,
    fs::File,
    io::{stdin, stdout, BufReader, BufWriter, ErrorKind, Read, Write},
};
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg};

#[derive(Clone, Copy, PartialEq)]
struct Size {
    width: u16,
    height: u16,
}

struct Crop {
    offset: u16,
    size: u16,
}

fn main() {

    let matches = app_from_crate!()
        .arg(Arg::with_name("crop-width")
            .long("crop-width")
            .short("w")
            .value_name("PIXELS")
            .help("Width to crop each subtitle frame to")
            .takes_value(true)
            .required(false)
            .validator(|value| {
                if value.parse::<usize>().is_ok() {
                    Ok(())
                } else {
                    Err("must be an unsigned integer".to_string())
                }
            })
        )
        .arg(Arg::with_name("crop-height")
            .long("crop-height")
            .short("h")
            .value_name("PIXELS")
            .help("Height to crop each subtitle frame to")
            .takes_value(true)
            .required(false)
            .validator(|value| {
                if value.parse::<usize>().is_ok() {
                    Ok(())
                } else {
                    Err("must be an unsigned integer".to_string())
                }
            })
        )
        .arg(Arg::with_name("crop-x")
            .long("crop-x")
            .short("x")
            .value_name("PIXELS")
            .help("Horizontal offset of the cropped frame")
            .takes_value(true)
            .required(false)
            .requires("crop-width")
            .validator(|value| {
                if value.parse::<usize>().is_ok() {
                    Ok(())
                } else {
                    Err("must be an unsigned integer".to_string())
                }
            })
        )
        .arg(Arg::with_name("crop-y")
            .long("crop-y")
            .short("y")
            .value_name("PIXELS")
            .help("Vertical offset of the cropped frame")
            .takes_value(true)
            .required(false)
            .requires("crop-height")
            .validator(|value| {
                if value.parse::<usize>().is_ok() {
                    Ok(())
                } else {
                    Err("must be an unsigned integer".to_string())
                }
            })
        )
        .arg(Arg::with_name("margin")
            .long("margin")
            .short("m")
            .value_name("PIXELS")
            .help("Minimum margin around the screen border to enforce")
            .takes_value(true)
            .required(false)
            .default_value("30")
            .validator(|value| {
                if value.parse::<usize>().is_ok() {
                    Ok(())
                } else {
                    Err("must be an unsigned integer".to_string())
                }
            })
        )
        .arg(Arg::with_name("lum-scale")
            .long("lum-scale")
            .short("l")
            .value_name("FACTOR")
            .help("Scales the gamma brightness of the subtitles by the specified factor")
            .takes_value(true)
            .required(false)
            .validator(|value| {
                let ref_white = value.parse::<f64>();
                if ref_white.is_err() {
                    return Err("Must be a floating point value".to_string())
                }
                let ref_white_value = ref_white.unwrap();
                if !ref_white_value.is_normal() {
                    return Err("Must be a normal number".to_string())
                }
                if !ref_white_value.is_sign_positive() {
                    return Err("Must be a positive number".to_string())
                }
                Ok(())
            })
        )
        .arg(Arg::with_name("input")
            .index(1)
            .value_name("INPUT-FILE")
            .help("Input PGS file; use - for STDIN")
            .required(true)
        )
        .arg(Arg::with_name("output")
            .index(2)
            .value_name("OUTPUT-FILE")
            .help("Output PGS file; use - for STDOUT")
            .required(true)
        )
        .after_help(format!("This utility will crop PGS subtitles found in Blu-ray discs so \
            that they can match any cropping that has been done to the main video stream, \
            thereby preventing the subtitles from appearing squished or distorted by the \
            player.\n\n\
            Copyright © 2021 William Swartzendruber\n\
            Licensed under the Mozilla Public License 2.0\n\
            <{}>", env!("CARGO_PKG_REPOSITORY")).as_str())
        .get_matches();
    let crop_width = match matches.value_of("crop-width") {
        Some(cw) => Some(cw.parse::<u16>().unwrap()),
        None => None
    };
    let crop_height = match matches.value_of("crop-height") {
        Some(ch) => Some(ch.parse::<u16>().unwrap()),
        None => None
    };
    let crop_x = match matches.value_of("crop-x") {
        Some(cx) => Some(cx.parse::<u16>().unwrap()),
        None => None
    };
    let crop_y = match matches.value_of("crop-y") {
        Some(cy) => Some(cy.parse::<u16>().unwrap()),
        None => None
    };
    let margin = matches.value_of("margin").unwrap().parse::<u16>().unwrap();
    let lum_scale = match matches.value_of("lum-scale") {
        Some(factor) => Some(factor.parse::<f64>().unwrap()),
        None => None,
    };
    let input_value = matches.value_of("input").unwrap();
    let (mut stdin_read, mut file_read);
    let mut input = BufReader::<&mut dyn Read>::new(
        if input_value == "-" {
            stdin_read = stdin();
            &mut stdin_read
        } else {
            file_read = File::open(input_value)
                .expect("Could not open input file for writing.");
            &mut file_read
        }
    );
    let output_value = matches.value_of("output").unwrap();
    let (mut stdout_write, mut file_write);
    let mut output = BufWriter::<&mut dyn Write>::new(
        if output_value == "-" {
            stdout_write = stdout();
            &mut stdout_write
        } else {
            file_write = File::create(output_value)
                .expect("Could not open output file for writing.");
            &mut file_write
        }
    );
    let mut screen_size = None;
    let mut width_crop = None;
    let mut height_crop = None;

    loop {

        let mut objects = HashMap::<u16, Object>::new();

        match input.read_display_set() {
            Ok(mut display_set) => {

                //
                // VALIDATE/SET SCREEN SIZE
                //

                let ds_size = Size {
                    width: display_set.width,
                    height: display_set.height,
                };

                match screen_size {
                    Some(ss) => {
                        if ds_size != ss {
                            panic!(
                                "Inconsistent screen size encountered: {}x{}",
                                ds_size.width,
                                ds_size.height,
                            )
                        }
                    }
                    None => {
                        eprintln!("Existing resolution: {}x{}", ds_size.width, ds_size.height);
                        screen_size = Some(ds_size);
                        width_crop = to_crop(ds_size.width, crop_width, crop_x);
                        height_crop = to_crop(ds_size.height, crop_height, crop_y);
                    }
                }

                //
                // UPDATE OBJECTS & WINDOWS
                //

                if display_set.composition.state == CompositionState::EpochStart
                    || display_set.composition.state == CompositionState::AcquisitionPoint {
                    objects.clear();
                }

                for (vid, object) in &display_set.objects {
                    objects.insert(vid.id, object.clone());
                }

                //
                // UDPATE SCREEN DIMENSIONS
                //

                match &width_crop {
                    Some(wc) => {
                        display_set.width = wc.size;
                        for window in display_set.windows.values_mut() {
                            window.x = new_item_offset(
                                wc.size, wc.offset, window.width, window.x, margin
                            );
                        }
                        for (cid, co) in &mut display_set.composition.objects {
                            match objects.get(&cid.object_id) {
                                Some(object) => {
                                    co.x = new_item_offset(
                                        wc.size, wc.offset, object.width, co.x, margin
                                    );
                                }
                                None =>
                                {
                                    panic!("Object referenced by composition not found.")
                                }
                            }
                        }
                    }
                    None => {
                    }
                }

                match &height_crop {
                    Some(hc) => {
                        display_set.height = hc.size;
                        for window in display_set.windows.values_mut() {
                            window.y = new_item_offset(
                                hc.size, hc.offset, window.height, window.y, margin
                            );
                        }
                        for (cid, co) in &mut display_set.composition.objects {
                            match objects.get(&cid.object_id) {
                                Some(object) => {
                                    co.y = new_item_offset(
                                        hc.size, hc.offset, object.height, co.y, margin
                                    );
                                }
                                None =>
                                {
                                    panic!("Object referenced by composition not found.")
                                }
                            }
                        }
                    }
                    None => {
                    }
                }

                //
                // LUMINOSITY SCALING
                //

                match lum_scale {
                    Some(factor) => {
                        for palette in display_set.palettes.values_mut() {
                            for entry in palette.entries.values_mut() {
                                let mut rgb = rgb_pixel(
                                    YcbcrPixel { y: entry.y, cb: entry.cb, cr: entry.cr }
                                );
                                rgb.red *= factor;
                                rgb.green *= factor;
                                rgb.blue *= factor;
                                let ycbcr = ycbcr_pixel(rgb);
                                entry.y = ycbcr.y;
                                entry.cb = ycbcr.cb;
                                entry.cr = ycbcr.cr;
                            }
                        }
                    }
                    None => {
                    }
                }

                if let Err(err) = output.write_display_set(display_set) {
                    panic!("Could not write display set to output stream: {:?}", err)
                }
            }
            Err(err) => {
                match err {
                    DisplaySetReadError::ReadError { source } => {
                        match source {
                            SegmentReadError::IoError { source } => {
                                if source.kind() != ErrorKind::UnexpectedEof {
                                    panic!("Could not read segment due to IO error: {}", source)
                                }
                            }
                            _ => {
                                panic!(
                                    "Could not read display set due to segment error: {}",
                                    source,
                                )
                            }
                        }
                    }
                    _ => panic!("Could not read display set due to bitstream error: {}", err)
                }
                break
            }
        };
    }
}

fn to_crop(old_size: u16, new_size: Option<u16>, offset: Option<u16>) -> Option<Crop> {
    match new_size {
        Some(ns) => {
            match offset {
                Some(o) => {
                    Some(
                        Crop {
                            size: ns,
                            offset: o,
                        }
                    )
                }
                None => {
                    Some(
                        Crop {
                            size: ns,
                            offset: (old_size - ns) / 2,
                        }
                    )
                }
            }
        }
        None => {
            None
        }
    }
}

fn new_item_offset(
    screen_size: u16,
    screen_offset: u16,
    item_size: u16,
    item_offset: u16,
    margin: u16,
) -> u16 {
    if item_size > screen_size - 2 * margin {
        panic!("Object does not fit within new screen dimensions.")
    } else if item_offset < screen_offset + margin {
        margin
    } else {
        if item_offset - screen_offset + item_size > screen_size - margin {
            screen_size - item_size - margin
        } else {
            item_offset - screen_offset
        }
    }
}
