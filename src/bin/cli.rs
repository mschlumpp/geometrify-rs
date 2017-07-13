/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

extern crate geometrify;
extern crate image;
extern crate clap;
extern crate pbr;

use clap::{Arg, App, AppSettings};

use geometrify::{ProgressReporter, RandomPointGenerator, Filter};
use geometrify::geometrify::Geometrify;

use image::open;
use std::io::Stdout;
use std::path::Path;

use pbr::ProgressBar;

struct PbrProgressReporter {
    bar: Option<ProgressBar<Stdout>>,
}

impl PbrProgressReporter {
    fn new() -> PbrProgressReporter {
        PbrProgressReporter { bar: None }
    }
}

impl ProgressReporter for PbrProgressReporter {
    fn init(&mut self, steps: u64) {
        let mut progress = ProgressBar::new(steps);
        progress.format("|#--|");
        self.bar = Some(progress);
    }

    fn step(&mut self) {
        let ref mut bar = self.bar
            .as_mut()
            .expect("ProgressReporter was not initialized");
        bar.inc();
    }

    fn finish(&mut self) {
        {
            let ref mut bar = self.bar
                .as_mut()
                .expect("ProgressReporter was not initialized");
            bar.finish();
        }
        self.bar = None;
    }
}

fn main() {
    let matches = App::new("Geometrify Filter")
        .version("1.0")
        .setting(AppSettings::ColorAlways)
        .arg(
            Arg::with_name("INPUT")
                .required(true)
                .help("Input file")
                .index(1),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .required(true)
                .help("Output file")
                .index(2),
        )
        .arg(
            Arg::with_name("samples")
                .short("s")
                .long("samples")
                .help("Number of primitives to select from")
                .default_value("50")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("iterations")
                .short("i")
                .long("iterations")
                .default_value("100")
                .help("Number of primitives to place")
                .takes_value(true),
        )
        .get_matches();


    let source = open(&Path::new(
        matches.value_of("INPUT").expect("expected input file"),
    )).expect("Can't open source file");
    let sourcebuf = source.to_rgba();


    let pointgen = Box::new(RandomPointGenerator::new());
    let mut filter = Geometrify::new(
        pointgen,
        matches
            .value_of("iterations")
            .unwrap_or("100")
            .parse::<u32>()
            .expect("invalid iterations parameter"),
        matches
            .value_of("samples")
            .unwrap_or("50")
            .parse::<u32>()
            .expect("invalid samples parameter"),
    );

    let mut progress = PbrProgressReporter::new();
    let outputbuf = filter.apply(&sourcebuf, &mut progress);

    outputbuf
        .save(&Path::new(
            matches.value_of("OUTPUT").expect("expected output file"),
        ))
        .expect("Can't save image");
}
