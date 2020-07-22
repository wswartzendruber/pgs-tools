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
use clap::{Arg, App};

fn main()  {

    let matches = App::new("pgsscale")
        .version("1.0.0")
        .author("William Swartzendruber")
        .about("Crops PGS subtitles")
        .arg(
            Arg::with_name("crop_width")
                .short("w")
                .long("crop_width")
                .help("the width to crop to")
                .takes_value(true)
                .required(true)
        )
        .arg(
            Arg::with_name("crop_height")
                .short("h")
                .long("crop_height")
                .help("the height to crop to")
                .takes_value(true)
                .required(true)
        )
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .help("input PGS file, or - for STDIN")
                .takes_value(true)
                .required(true)
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("output PGS file, or - for STDIN")
                .takes_value(true)
                .required(true)
        )
        .get_matches();

    let crop_width = matches.value_of("crop_width").unwrap().parse::<u16>().expect(
        "Invalid crop-width value, which must be a 16-bit unsigned integer."
    );
    let crop_height = matches.value_of("crop_height").unwrap().parse::<u16>().expect(
        "Invalid crop-height value, which must be a 16-bit unsigned integer."
    );

    //
    // INPUT/OUTPUT
    //

    let input_str = matches.value_of("input").unwrap();
    let output_str = matches.value_of("output").unwrap();

    let mut input: BufReader<Box<dyn Read>> = BufReader::new(
        if input_str == "-" {
            Box::new(stdin())
        } else {
            Box::new(
                File::open(&input_str).expect("Could not open input file for writing.")
            )
        }
    );
    let mut output: BufWriter<Box<dyn Write>> = BufWriter::new(
        if output_str == "-" {
            Box::new(stdout())
        } else {
            Box::new(
                File::create(&output_str).expect("Could not open output file for writing.")
            )
        }
    );

    //
    // PROCESSING
    //

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
