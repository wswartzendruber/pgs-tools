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

type Pixel = (f64, f64, f64);

fn main()  {

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
        .arg(Arg::with_name("pq-to-hlg")
            .long("pq-to-hlg")
            .short("p")
            .help("Enable PQ to HLG conversion of subtitle colors")
            .takes_value(false)
        )
        .arg(Arg::with_name("ref-white")
            .long("ref-white")
            .short("r")
            .value_name("NITS")
            .help("Brightness of the video's reference white level")
            .takes_value(true)
            .requires("pq-to-hlg")
            .required(false)
            .default_value("203")
            .validator(|value| {
                let ref_white = value.parse::<f64>();
                if ref_white.is_err() {
                    return Err("must be a floating point value".to_string())
                }
                let ref_white_value = ref_white.unwrap();
                if !ref_white_value.is_normal() {
                    return Err("must be a normal number".to_string())
                }
                if !ref_white_value.is_sign_positive() {
                    return Err("must be a positive number".to_string())
                }
                Ok(())
            })
        )
        .arg(Arg::with_name("input")
            .index(1)
            .value_name("INPUT-FILE")
            .help("Input PGS file")
            .required(true)
        )
        .arg(Arg::with_name("output")
            .index(2)
            .value_name("OUTPUT-FILE")
            .help("Output PGS file")
            .required(true)
        )
        .after_help("This utility will crop PGS subtitles found in Blu-ray discs so that they \
            can match any cropping that has been done to the main video stream, thereby \
            preventing the subtitles from appearing squished or distorted by the player. \
            Subtitles from 4K UltraHD discs can optionally be tone mapped to appropriate HLG \
            levels.")
        .get_matches();

    let crop_width = matches.value_of("crop_width").unwrap().parse::<u16>().unwrap();
    let crop_height = matches.value_of("crop_height").unwrap().parse::<u16>().unwrap();
    let pq_to_hlg = matches.is_present("pq_to_hlg");
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
            SegBody::PalDef(pds) => {
                if pq_to_hlg {
                    for entry in pds.entries.iter_mut() {
                        let (y, cb, cr) = ycbcr(pq_hlg_ootf(rgb((
                            entry.y as f64 / 255.0,
                            (entry.cb as f64 - 128.0) / 127.0,
                            (entry.cr as f64 - 128.0) / 127.0,
                        ))));
                        entry.y = (y * 255.0) as u8;
                        entry.cb = (cb * 127.0 + 128.0) as u8;
                        entry.cr = (cr * 127.0 + 128.0) as u8;
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

fn pq_hlg_ootf(pixel: Pixel) -> Pixel {

    //
    // The BBC R&D Method of PQ to HLG Transcoding
    //

    let rd = (pq_eotf(pixel.0) * 10.0).min(1.0);
    let gd = (pq_eotf(pixel.1) * 10.0).min(1.0);
    let bd = (pq_eotf(pixel.2) * 10.0).min(1.0);
    let yd = 0.2627 * rd + 0.6780 * gd + 0.0593 * bd;
    let yg = yd.powf(-0.166666667);
    let rs = rd * yg;
    let gs = gd * yg;
    let bs = bd * yg;

    (hlg_oetf(rs), hlg_oetf(gs), hlg_oetf(bs))
}

fn pq_eotf(e: f64) -> f64 {

    //
    // ITU-R BT.2100-2
    // Table 4
    //

    (
        (e.powf(0.012683313515655966) - 0.8359375).max(0.0)
        /
        (18.8515625 - 18.6875 * e.powf(0.012683313515655966))
    )
    .powf(6.277394636015326)
}

fn hlg_oetf(o: f64) -> f64 {

    //
    // ITU-R BT.2100-2
    // Table 5
    //

    if o < 0.08333333333333333 {
        (3.0 * o).sqrt()
    } else {
        0.17883277 * (12.0 * o - 0.28466892).ln() + 0.559910729529562
    }
}

fn rgb(ycbcr: Pixel) -> Pixel {
    (
        ycbcr.0 + 1.4746 * ycbcr.2,
        ycbcr.0 + -0.1645531268436578 * ycbcr.1 + -0.5713531268436578 * ycbcr.2,
        ycbcr.0 + 1.8814 * ycbcr.1,
    )
}

fn ycbcr(rgb: Pixel) -> Pixel {
    (
        0.2627 * rgb.0 + 0.6780 * rgb.1 + 0.0593 * rgb.2,
        -0.13963006271925163 * rgb.0 + -0.3603699372807484 * rgb.1 + 0.5 * rgb.2,
        0.5 * rgb.0 + -0.45978570459785706 * rgb.1 + -0.04021429540214295 * rgb.2,
    )
}
