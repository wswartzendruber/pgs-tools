/*
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a
 * copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 William Swartzendruber
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use pgs::{
    ts_to_timestamp,
    segment::{
        CompositionState,
        ReadSegmentExt,
        Segment,
        Sequence,
        ReadError,
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
        .after_help(format!("This utility will dump PGS subtitle bitstream data.\n\n\
            Copyright © 2021 William Swartzendruber\n\
            Licensed under the Mozilla Public License 2.0\n\
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

    eprintln!("Iterating through PGS segments...");

    //
    // READ
    //

    loop {

        match input.read_segment() {
            Ok(segment) => {
                match segment {
                    Segment::PresentationComposition(pcs) => {
                        println!(
                            "presentation_composition_segment({})",
                            ts_to_timestamp(pcs.pts),
                        );
                        println!("  composition_number = {}", pcs.composition_number);
                        println!("  composition_state = {}", match pcs.composition_state {
                            CompositionState::EpochStart => "EPOCH_START",
                            CompositionState::Normal => "NORMAL_CASE",
                            CompositionState::AcquisitionPoint => "ACQUISITION_POINT",
                        });
                        match pcs.palette_update_id {
                            Some(pal_id) => println!("  palette_update_id = {}", pal_id),
                            None => (),
                        }
                        for comp_obj in pcs.composition_objects.iter() {
                            println!("  window_information");
                            println!("    object_id = {}", comp_obj.object_id);
                            println!("    window_id = {}", comp_obj.window_id);
                            println!("    object_horizontal_position = {}", comp_obj.x);
                            println!("    object_vertical_position = {}", comp_obj.y);
                            match &comp_obj.crop {
                                Some(crop) => {
                                    println!(
                                        "    object_cropping_horizontal_position = {}",
                                        crop.x,
                                    );
                                    println!(
                                        "    object_cropping_vertical_position = {}",
                                        crop.y,
                                    );
                                    println!("    object_cropping_width = {}", crop.width);
                                    println!("    object_crooping_height = {}", crop.height);
                                }
                                None => { }
                            }
                        }
                    }
                    Segment::WindowDefinition(wds) => {
                        println!("window_definition_segment({})", ts_to_timestamp(wds.pts));
                        for wd in wds.windows.iter() {
                            println!("  window_id = {}", wd.id);
                            println!("  window_horizontal_position = {}", wd.x);
                            println!("  window_vertical_position = {}", wd.y);
                            println!("  window_width = {}", wd.width);
                            println!("  window_height = {}", wd.height);
                        }

                    }
                    Segment::ObjectDefinition(ods) => {
                        println!("object_definition_segment({})", ts_to_timestamp(ods.pts));
                        println!("  object_id = {}", ods.id);
                        println!("  object_version_number = {}", ods.version);
                        println!("  object_sequence = {}", match ods.sequence {
                            Sequence::Single => "SINGLE",
                            Sequence::First => "FIRST",
                            Sequence::Last => "LAST",
                        });
                        println!("  object_width = {}", ods.width);
                        println!("  object_height = {}", ods.height);
                        println!("  object_data = [{} lines]", ods.lines.len());
                    }
                    Segment::PaletteDefinition(pds) => {
                        println!("palette_definition_segment({})", ts_to_timestamp(pds.pts));
                        println!("  palette_id = {}", pds.id);
                        println!("  palette_version_number = {}", pds.version);
                        for pe in pds.entries.iter() {
                            println!("  palette_entry");
                            println!("    y_value = {}", pe.y);
                            println!("    cb_value = {}", pe.cb);
                            println!("    cr_value = {}", pe.cr);
                            println!("    t_value = {}", pe.alpha);
                        }
                    }
                    Segment::End(es) => {
                        println!("end_segment({})", ts_to_timestamp(es.pts));
                        println!();
                    }
                }
            }
            Err(err) => {
                match err {
                    ReadError::IoError { source } => {
                        if source.kind() != ErrorKind::UnexpectedEof {
                            panic!("Could not read segment due to IO error: {}", source)
                        }
                    }
                    _ => panic!("Could not read segment due to bitstream error: {}", err)
                }
                break
            }
        };
    }
}
