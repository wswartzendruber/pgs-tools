/*
 * Copyright 2020 William Swartzendruber
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 * except in compliance with the License. You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software distributed under the
 * License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
 * either express or implied. See the License for the specific language governing permissions
 * and limitations under the License.
 */

pub mod pgs;

use pgs::{
    SegBody,
    read::{ReadSegExt, SegReadError},
    write::WriteSegExt,
};
use std::{
    fs::File,
    io::{stdin, stdout, BufReader, BufWriter, ErrorKind, Read, Write},
};
use clap::{crate_version, Arg, App};

type Size = (u16, u16);

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
            .help("Input PGS file; use - for STDIN")
            .required(true)
        )
        .arg(Arg::with_name("output")
            .index(2)
            .value_name("OUTPUT-FILE")
            .help("Output PGS file; use - for STDOUT")
            .required(true)
        )
        .after_help("This utility will crop PGS subtitles found in Blu-ray discs so that they \
            can match any cropping that has been done to the main video stream, thereby \
            preventing the subtitles from appearing squished or distorted by the player.")
        .get_matches();
    let crop_width = matches.value_of("crop-width").unwrap().parse::<u16>().unwrap();
    let crop_height = matches.value_of("crop-height").unwrap().parse::<u16>().unwrap();
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
    let mut sizes = Vec::<Size>::new();
    let mut size;

    'segs: loop {

        let mut seg = match input.read_seg() {
            Ok(seg) => seg,
            Err(err) => {
                match err {
                    SegReadError::IoError { source } => {
                        if source.kind() != ErrorKind::UnexpectedEof {
                            panic!("Could not read segment due to IO error: {}", source)
                        }
                    }
                    _ => panic!("Could not read segment due to bitstream error: {}", err)
                }
                break 'segs
            },
        };

        match &mut seg.body {
            SegBody::PresComp(pcs) => {
                size = (pcs.width, pcs.height);
                if !sizes.contains(&size) {
                    eprintln!("New resolution encountered: {}x{}", size.0, size.1);
                    sizes.push(size);
                }
                pcs.width = crop_width;
                pcs.height = crop_height;
                for comp_obj in pcs.comp_objs.iter_mut() {
                    comp_obj.x = cropped_offset(comp_obj.x, size.0, crop_width);
                    comp_obj.y = cropped_offset(comp_obj.y, size.1, crop_height);
                    match &mut comp_obj.crop {
                        Some(crop) => {
                            crop.x = cropped_offset(crop.x, size.0, crop_width);
                            crop.y = cropped_offset(crop.y, size.1, crop_height);
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
