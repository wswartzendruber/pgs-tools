/*
 * SPDX-FileCopyrightText: 2021 William Swartzendruber <wswartzendruber@gmail.com>
 *
 * SPDX-License-Identifier: OSL-3.0
 */

use pgs::{
    displayset::{
        ReadDisplaySetExt,
        ReadError as DisplaySetReadError,
    },
    segment::{
        ReadError as SegmentReadError,
    },
};
use std::{
    fs::File,
    io::{stdin, BufReader, ErrorKind, Read},
};
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg};

fn main() {

    let matches = app_from_crate!()
        .arg(Arg::with_name("input")
            .index(1)
            .value_name("INPUT-FILE")
            .help("Input PGS file; use - for STDIN")
            .required(true)
        )
        .after_help(format!("This utility will test PGS subtitles.\n\n\
            Copyright © 2021 William Swartzendruber\n\
            Licensed under the Open Software License version 3.0\n\
            <{}>", env!("CARGO_PKG_REPOSITORY")).as_str())
        .get_matches();
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

    eprintln!("Iterating through PGS display sets...");

    //
    // READ
    //

    loop {

        match input.read_display_set() {
            Ok(_) => {
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
