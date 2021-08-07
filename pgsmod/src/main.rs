/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: OSL-3.0
 */

#[cfg(test)]
mod tests;

mod rgb;

use pgs::{
    ts_to_timestamp,
    displayset::{
        ReadDisplaySetExt,
        ReadError as DisplaySetReadError,
        WriteDisplaySetExt,
    },
    segment::{
        ReadError as SegmentReadError,
    },
};
use rgb::{rgb_pixel, ycbcr_pixel, YcbcrPixel};
use std::{
    fs::File,
    io::{stdin, stdout, BufReader, BufWriter, ErrorKind, Read, Write},
};
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg};

#[derive(Clone, Copy, PartialEq)]
struct Size {
    width: u16,
    height: u16,
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
            Licensed under the Open Software License version 3.0\n\
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
    let mut screen_sizes = Vec::<Size>::new();

    loop {

        match &mut input.read_display_set() {
            Ok(display_set) => {

                let screen_size = Size {
                    width: display_set.width,
                    height: display_set.height,
                };

                if !screen_sizes.contains(&screen_size) {
                    eprintln!(
                        "New resolution encountered: {}x{}",
                        screen_size.width, screen_size.height,
                    );
                    screen_sizes.push(screen_size);
                }

                match crop_width {
                    Some(cw) => display_set.width = cw,
                    None => (),
                }
                match crop_height {
                    Some(ch) => display_set.height = ch,
                    None => (),
                }

                for (cid, composition_object) in display_set.composition.objects.iter_mut() {

                    let object_sizes = display_set.objects.iter()
                        .filter(|(object_vid, _)| object_vid.id == cid.object_id)
                        .map(|(_, object)| Size { width: object.width, height: object.height })
                        .collect::<Vec<Size>>();
                    let object_width = object_sizes.iter()
                        .map(|size| size.width)
                        .max()
                        .unwrap();
                    let object_height = object_sizes.iter()
                        .map(|size| size.height)
                        .max()
                        .unwrap();

                    match crop_width {
                        Some(cw) => {
                            composition_object.x = new_item_offset(
                                screen_size.width,
                                match crop_x {
                                    Some(cx) => cx,
                                    None => (screen_size.width - cw) / 2,
                                },
                                object_width,
                                composition_object.x,
                                margin,
                            );
                        }
                        None => {
                        }
                    }
                    match crop_height {
                        Some(ch) => {
                            composition_object.y = new_item_offset(
                                screen_size.height,
                                match crop_y {
                                    Some(cy) => cy,
                                    None => (screen_size.height - ch) / 2,
                                },
                                object_height,
                                composition_object.y,
                                margin,
                            );
                        }
                        None => {
                        }
                    }
                }

                for window in display_set.windows.values_mut() {
                    match crop_width {
                        Some(cw) => {
                            window.x = new_item_offset(
                                screen_size.width,
                                match crop_x {
                                    Some(cx) => cx,
                                    None => (screen_size.width - cw) / 2,
                                },
                                window.width,
                                window.x,
                                margin,
                            );
                        }
                        None => {
                        }
                    }
                    match crop_height {
                        Some(ch) => {
                            window.y = new_item_offset(
                                screen_size.height,
                                match crop_y {
                                    Some(cy) => cy,
                                    None => (screen_size.height - ch) / 2,
                                },
                                window.height,
                                window.y,
                                margin,
                            );
                        }
                        None => {
                        }
                    }
                }

                for (window_id_1, window_1) in display_set.windows.iter() {
                    for (window_id_2, window_2) in display_set.windows.iter() {
                        if window_id_1 != window_id_2 {

                            let window_1_ex = window_1.x + window_1.width;
                            let window_1_ey = window_1.y + window_1.height;

                            if window_1.x <= window_2.x && window_2.x <= window_1_ex
                                && window_1.y <= window_2.y && window_2.y <= window_1_ey {
                                panic!(
                                    "window collision detected at {}",
                                    ts_to_timestamp(display_set.pts),
                                )
                            }
                        }
                    }
                }

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
                    DisplaySetReadError::SegmentError { source } => {
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
