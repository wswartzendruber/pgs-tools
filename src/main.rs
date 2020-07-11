/*
 * Copyright 2020 William Swartzendruber
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a
 * copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

pub mod pgs;

use std::fs::File;
use std::env;
use pgs::*;
use pgs::read::*;

fn main()  {

    let args: Vec<String> = env::args().collect();
    let mut file = File::open(&args[1]).expect("could not open input file");

    loop {
        match file.read_seg().unwrap().body {
            SegBody::PresComp(pcs) => println!(
                "PresentationCompositionSegment: {}x{}",
                pcs.width,
                pcs.height,
            ),
            SegBody::WinDef(_) => println!("WindowDefinitionSegment"),
            SegBody::PalDef(_) => println!("PaletteDefinitionSegment"),
            SegBody::ObjDef(_) => println!("ObjectDefinitionSegment"),
            SegBody::End(_) => println!("EndSegment"),
        }
    }
}
