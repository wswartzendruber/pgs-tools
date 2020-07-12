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
    env,
    fs::File,
    io::{stdin, stdout, BufReader, BufWriter, Read, Write},
    process::exit,
};

fn main()  {

    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        eprintln!("ERROR: Incorrect number of arguments specificed.");
        eprintln!("USAGE: pgsscale [crop-width] [crop-height] [input] [output]");
        eprintln!("  crop-width  - Cropping width of the video stream in pixels.");
        eprintln!("  crop-height - Cropping height of the video stream in pixels.");
        eprintln!("  input       - PGS input file; use - for STDIN.");
        eprintln!("  output      - PGS output file; use - for STDOUT.");
        exit(1);
    }

    let crop_width = args[1].parse::<u16>().expect(
        "Invalid crop-width value, which must be a 16-bit unsigned integer."
    );
    let crop_height = args[2].parse::<u16>().expect(
        "Invalid crop-height value, which must be a 16-bit unsigned integer."
    );

    //
    // INPUT/OUTPUT
    //

    let mut input: BufReader<Box<dyn Read>> = BufReader::new(
        if args[3] == "-" {
            Box::new(stdin())
        } else {
            Box::new(File::open(&args[3]).expect("Could not open input file for writing."))
        }
    );
    let mut output: BufWriter<Box<dyn Write>> = BufWriter::new(
        if args[4] == "-" {
            Box::new(stdout())
        } else {
            Box::new(File::create(&args[4]).expect("Could not open output file for writing."))
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
                if crop_width > width {
                    panic!("crop_width is greater than width={} defined by PCS.", width);
                }
                if crop_height > height {
                    panic!("crop_height is greater than height={} defines by PCS.", height);
                }
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
