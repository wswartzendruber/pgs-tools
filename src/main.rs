/*
 * Copyright 2020 William Swartzendruber
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a
 * copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

pub mod pgs;

use pgs::{
    SegBody,
    read::ReadSegExt,
    write::WriteSegExt,
};
use std::{
    fs::File,
    io::{stdin, stdout, BufReader, BufWriter, Read, Write},
};
use clap::{crate_version, Arg, App};

fn main() {

    let matches = App::new("PGSScale")
        .version(crate_version!())
        .about("Scales PGS subtitles")
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
        .arg(Arg::with_name("input")
            .index(1)
            .value_name("INPUT-FILE")
            .help("Input PGS file; use - for STDIN.")
            .required(true)
        )
        .arg(Arg::with_name("output")
            .index(2)
            .value_name("OUTPUT-FILE")
            .help("Output PGS file; use - for STDOUT.")
            .required(true)
        )
        .after_help("This utility will crop PGS subtitles found in Blu-ray discs so that they \
            can match any cropping that has been done to the main video stream, thereby \
            preventing the subtitles from appearing squished or distorted by the player.")
        .get_matches();

    let crop_width = matches.value_of("crop_width").unwrap().parse::<u16>().unwrap();
    let crop_height = matches.value_of("crop_height").unwrap().parse::<u16>().unwrap();
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
    let mut width;
    let mut height;

    'segs: loop {

        let mut seg = match input.read_seg() {
            Ok(seg) => seg,
            Err(err) => {
                eprintln!("Could not read anymore segments: {:?}", err);
                break 'segs
            },
        };

        match &mut seg.body {
            SegBody::PresComp(pcs) => {
                width = pcs.width;
                height = pcs.height;
                pcs.width = crop_width;
                pcs.height = crop_height;
                for comp_obj in pcs.comp_objs.iter_mut() {
                    comp_obj.x = cropped_offset(comp_obj.x, width, crop_width);
                    comp_obj.y = cropped_offset(comp_obj.y, height, crop_height);
                    match &mut comp_obj.crop {
                        Some(crop) => {
                            crop.x = cropped_offset(crop.x, width, crop_width);
                            crop.y = cropped_offset(crop.y, height, crop_height);
                        },
                        None => (),
                    }
                }
            },
            _ => ()
        }

        if let Err(err) = output.write_seg(&seg) {
            panic!("Could not write frame to output stream: {:?}", err)
        }
    }

    output.flush().expect("Could not flush output stream.");
}

fn cropped_offset(offset: u16, size: u16, crop: u16) -> u16 {
    offset - (size - crop) / 2
}
