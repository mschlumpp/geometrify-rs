extern crate geometrify;
extern crate image;

use geometrify::{RandomPointGenerator, Geometrify};

use image::open;
use std::path::Path;

fn main() {
    let source = open(&Path::new("/home/marco/AndroidSync/Universität/SWT_I/Blatt_2/dices_alpha.png")).expect("Can't open source file");
    // let source = open(&Path::new("/home/marco/AndroidSync/Universität/SWT_I/Blatt_2/walter_no_alpha.png")).expect("Can't open source file");
    // let source = open(&Path::new("/home/marco/Downloads/windows_xp_bliss-wide.jpg")).expect("Can't open source file");
    let sourcebuf = source.to_rgba();

    let mut filter = Geometrify::new(RandomPointGenerator::new(sourcebuf.width() as i32, sourcebuf.height() as i32));


    let outputbuf = filter.apply(sourcebuf, 1000, 30);

    outputbuf.save(&Path::new("/tmp/hello.png")).expect("Can't save image");
}
