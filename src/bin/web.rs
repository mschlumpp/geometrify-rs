/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate geometrify;
extern crate image;

use rocket::Data;

use std::io::Read;

use geometrify::{Filter, SilentProgressReporter, RandomPointGenerator};
use geometrify::geometrify::Geometrify;

#[get("/")]
fn index() -> &'static str {
    "Hello world!"
}

#[post("/geometrify", format = "image/png", data = "<data>")]
fn geometrify_process(data: Data) -> &'static str {
    let mut buf = vec![];
    data.open()
        .read_to_end(&mut buf)
        .expect("Can't read into memory");
    let image = image::load_from_memory(&buf)
        .expect("Can't load image")
        .to_rgba();
    drop(buf);

    let pointgen = Box::new(RandomPointGenerator::new());
    let mut filter = Geometrify::new(pointgen, 4, 6);

    let mut progress = SilentProgressReporter::new();
    let out = filter.apply(&image, &mut progress);

    "ok"
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, geometrify_process])
        .launch();
}
