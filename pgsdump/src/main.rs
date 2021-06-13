/*
 * Copyright © 2021 William Swartzendruber
 * Licensed under the Open Software License version 3.0
 */

use pgs::{
    CompState,
    ObjSeq,
    SegBody,
    read::{ReadSegExt, SegReadError},
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

    eprintln!("Iterating through PGS segments...");

    //
    // READ
    //

    loop {

        match input.read_seg() {
            Ok(seg) => {
                let timestamp = ts_to_timestamp(seg.pts);
                match seg.body {
                    SegBody::PresComp(pcs) => {
                        println!("presentation_composition_segment({})", timestamp);
                        println!("  composition_number = {}", pcs.comp_num);
                        println!("  composition_state = {}", match pcs.comp_state {
                            CompState::EpochStart => "EPOCH_START",
                            CompState::Normal => "NORMAL_CASE",
                            CompState::AcquisitionPoint => "ACQUISITION_POINT",
                        });
                        println!("  palette_update_flag = {}", match pcs.pal_update {
                            true => "TRUE",
                            false => "FALSE",
                        });
                        println!("  palette_id = {}", pcs.pal_id);
                        for comp_obj in pcs.comp_objs.iter() {
                            println!("  window_information");
                            println!("    object_id = {}", comp_obj.obj_id);
                            println!("    window_id = {}", comp_obj.win_id);
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
                    SegBody::WinDef(wds) => {
                        for wd in wds.iter() {
                            println!("window_definition_segment({})", timestamp);
                            println!("  window_id = {}", wd.id);
                            println!("  window_horizontal_position = {}", wd.x);
                            println!("  window_vertical_position = {}", wd.y);
                            println!("  window_width = {}", wd.width);
                            println!("  window_height = {}", wd.height);
                        }

                    }
                    SegBody::ObjDef(ods) => {
                        println!("object_definition_segment({})", timestamp);
                        println!("  object_id = {}", ods.id);
                        println!("  object_version_number = {}", ods.version);
                        match ods.seq {
                            Some(seq) => {
                                println!("  object_sequence = {}", match seq {
                                    ObjSeq::Last => "LAST",
                                    ObjSeq::First => "FIRST",
                                    ObjSeq::Both => "BOTH",
                                });
                            }
                            None => { }
                        }
                    }
                    SegBody::PalDef(pds) => {
                        println!("palette_definition_segment({})", timestamp);
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
                    SegBody::End => {
                        println!("end_segment({})", timestamp);
                        println!();
                    }
                }
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
    }

    fn ts_to_timestamp(ts: u32) -> String {

        let mut ms = ts / 90;
        let h = ms / 3_600_000;
        ms -= h * 3_600_000;
        let m = ms / 60_000;
        ms -= m * 60_000;
        let s = ms / 1_000;
        ms -= s * 1_000;

        format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms)
    }
}
