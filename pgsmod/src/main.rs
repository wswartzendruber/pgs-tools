/*
 * Copyright © 2021 William Swartzendruber
 * Licensed under the Open Software License version 3.0
 */

mod rgb;

use pgs::{
    Seg,
    SegBody,
    read::{ReadSegExt, SegReadError},
    write::WriteSegExt,
};
use rgb::{rgb_linear_pixel, ycbcr_gamma_pixel, YcbcrGammaPixel};
use std::{
    fs::File,
    io::{stdin, stdout, BufReader, BufWriter, ErrorKind, Read, Write},
};
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg};

#[derive(Eq, Hash, PartialEq)]
struct ObjHandle {
    comp_num: u16,
    obj_id: u16,
}

#[derive(Clone, Copy, PartialEq)]
struct Size {
    width: u16,
    height: u16,
}

const TONE_MAX_SDR: f64 = 1.0;
const TONE_MAX_PQ: f64 = 0.2705373206557394;
const TONE_MAX_HLG: f64 = 0.5013569413029385;

fn main() {

    let tone_maps = ["sdr", "pq", "hlg"];
    let matches = app_from_crate!()
        .arg(Arg::with_name("crop-width")
            .long("crop-width")
            .short("w")
            .value_name("PIXELS")
            .help("Width to crop each subtitle frame to")
            .takes_value(true)
            .required(true)
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
            .required(true)
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
        .arg(Arg::with_name("tone-map")
            .long("tone-map")
            .short("t")
            .help("Apply tone mapping")
            .takes_value(true)
            .required(false)
            .possible_values(&tone_maps)
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
    let crop_width = matches.value_of("crop-width").unwrap().parse::<u16>().unwrap();
    let crop_height = matches.value_of("crop-height").unwrap().parse::<u16>().unwrap();
    let margin = matches.value_of("margin").unwrap().parse::<u16>().unwrap();
    let tone_max = match matches.value_of("tone-map") {
        Some(max) => match max {
            "sdr" => Some(TONE_MAX_SDR),
            "pq" => Some(TONE_MAX_PQ),
            "hlg" => Some(TONE_MAX_HLG),
            _ => panic!("Invalid tone-map maximum selected."),
        }
        None => None
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
    let mut segs = Vec::<Seg>::new();

    eprintln!("Reading PGS segments into memory...");

    //
    // READ
    //

    loop {

        let seg = match input.read_seg() {
            Ok(seg) => {
                seg
            }
            Err(err) => {
                match err {
                    SegReadError::IoError { source } => {
                        if source.kind() != ErrorKind::UnexpectedEof {
                            panic!("Could not read segment due to IO error: {}", source)
                        }
                    }
                    _ => panic!("Could not read segment due to bitstream error: {}", err)
                }
                break
            }
        };

        segs.push(seg);
    }

    //
    // INVENTORY
    //

    let mut max_channel = 0.0_f64;

    eprintln!("Reading palette definitions...");

    for seg in segs.iter() {
        match &seg.body {
            SegBody::PalDef(pds) => {
                for pde in pds.entries.iter() {

                    let rgb = rgb_linear_pixel(
                        YcbcrGammaPixel { y: pde.y, cb: pde.cb, cr: pde.cr }
                    );

                    max_channel = max_channel.max(rgb.red).max(rgb.green).max(rgb.blue);
                }
            }
            _ => {
                ()
            }
        }
    }

    let tone_ratio = match tone_max {
        Some(x) => Some(max_channel / x),
        None => None,
    };
    let mut screen_sizes = Vec::<Size>::new();
    let mut screen_full_size = Size { width: 0, height: 0 };

    //
    // MODIFY
    //

    eprintln!("Performing modifications...");

    for seg in segs.iter_mut() {
        match &mut seg.body {
            SegBody::PresComp(pcs) => {
                screen_full_size = Size { width: pcs.width, height: pcs.height };
                if !screen_sizes.contains(&screen_full_size) {
                    eprintln!(
                        "New resolution encountered: {}x{}",
                        screen_full_size.width, screen_full_size.height,
                    );
                    screen_sizes.push(screen_full_size);
                }
                pcs.width = crop_width;
                pcs.height = crop_height;
            }
            SegBody::WinDef(wds) => {
                for wd in wds.iter_mut() {
                    wd.x = cropped_window_offset(
                        screen_full_size.width,
                        crop_width,
                        wd.width,
                        wd.x,
                        margin,
                    );
                    wd.y = cropped_window_offset(
                        screen_full_size.height,
                        crop_height,
                        wd.height,
                        wd.y,
                        margin,
                    );
                }
            }
            SegBody::PalDef(pds) => {
                match tone_ratio {
                    Some(x) => {
                        for pde in pds.entries.iter_mut() {
                            let mut rgb = rgb_linear_pixel(
                                YcbcrGammaPixel { y: pde.y, cb: pde.cb, cr: pde.cr }
                            );
                            rgb.red /= x;
                            rgb.green /= x;
                            rgb.blue /= x;
                            let ycbcr = ycbcr_gamma_pixel(rgb);
                            pde.y = ycbcr.y;
                            pde.cb = ycbcr.cb;
                            pde.cr = ycbcr.cr;
                        }
                    }
                    None => {
                        ()
                    }
                }
            }
            _ => ()
        }
    }

    //
    // WRITE
    //

    eprintln!("Writing modified segments...");

    for seg in segs {
        if let Err(err) = output.write_seg(&seg) {
            panic!("Could not write frame to output stream: {:?}", err)
        }
    }

    output.flush().expect("Could not flush output stream.");
}

fn cropped_window_offset(
    screen_full_size: u16,
    screen_crop_size: u16,
    window_size: u16,
    window_offset: u16,
    margin: u16,
) -> u16 {

    if window_size + 2 * margin > screen_crop_size {
        eprintln!("WARNING: Window cannot fit within new margins.");
        return 0
    }

    let new_offset = window_offset - (screen_full_size - screen_crop_size) / 2;

    match new_offset {
        o if o < margin =>
            margin,
        o if o + window_size + margin > screen_crop_size =>
            screen_crop_size - window_size - margin,
        _ =>
            new_offset,
    }
}
