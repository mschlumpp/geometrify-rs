extern crate image;
extern crate rand;
extern crate rayon;

use image::{Rgba, RgbaImage, Pixel};

use rand::Rng;

use rayon::prelude::*;

#[derive(Clone, Copy, Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn cross_product(&self, b: Point) -> i32 {
        self.x * b.y - self.y * b.x
    }
}

#[derive(Debug, Copy, Clone)]
struct BoundingBox {
    top_left: Point,
    bottom_right: Point,
}

#[derive(Debug, Copy, Clone)]
struct Triangle {
    a: Point,
    b: Point,
    c: Point,
    color: Option<Rgba<u8>>,
}

impl Triangle {
    fn new(a: Point, b: Point, c: Point) -> Triangle {
        Triangle {
            a: a,
            b: b,
            c: c,
            color: None,
        }
    }

    // fn set_color(&mut self, c: Color) {
    //     self.color = Some(c);
    // }

    // fn get_color(&)

    fn is_inside_primitive(&self, p: Point) -> bool {
        let spanA = Point { x: self.b.x - self.a.x, y: self.b.y - self.a.y };
        let spanB = Point { x: self.c.x - self.a.x, y: self.c.y - self.a.y };

        let q = Point { x: p.x - self.a.x, y: p.y - self.a.y };

        let s = q.cross_product(spanB) as f32 / spanA.cross_product(spanB) as f32;
        let t = spanA.cross_product(q) as f32 / spanA.cross_product(spanB) as f32;

        (s >= 0.0) && (t >= 0.0) && ((s + t) <= 1.0)
    }

    fn get_bounding_box(&self) -> BoundingBox {
        use std::cmp::{min, max};
        BoundingBox {
            top_left: Point {
                x: min(min(self.a.x, self.b.x), self.c.x),
                y: min(min(self.a.y, self.b.y), self.c.y),
            },
            bottom_right: Point {
                x: max(max(self.a.x, self.b.x), self.c.x),
                y: max(max(self.a.y, self.b.y), self.c.y),
            }
        }
    }
}

pub struct RandomPointGenerator {
    rng: Box<Rng>,
    width: i32,
    height: i32,
}

impl RandomPointGenerator {
    pub fn new(width: i32, height: i32) -> RandomPointGenerator {
        RandomPointGenerator {
            rng: Box::new(rand::thread_rng()),
            width: width,
            height: height,
        }
    }

    fn next_point(&mut self) -> Point {
        Point {
            x: self.rng.gen_range(0, self.width),
            y: self.rng.gen_range(0, self.height),
        }
    }
}

pub struct Geometrify {
    point_gen: RandomPointGenerator,
}

impl Geometrify {
    pub fn new(point_gen: RandomPointGenerator) -> Geometrify {
        Geometrify {
            point_gen: point_gen
        }
    }

    fn calculate_color(image: &RgbaImage, primitive: Triangle) -> Rgba<u8> {
        let bb = primitive.get_bounding_box();

        let mut count = 0u64;
        let mut cr = 0u64;
        let mut cg = 0u64;
        let mut cb = 0u64;
        let mut ca = 0u64;

        for y in bb.top_left.y..bb.bottom_right.y {
            for x in bb.top_left.x..bb.bottom_right.x {
                let (r, g, b, a) = image.get_pixel(x as u32, y as u32).channels4();
                cr += r as u64;
                cg += g as u64;
                cb += b as u64;
                ca += a as u64;
                count += 1;
            }
        }

        if count == 0 {
            Rgba::from_channels(255, 255, 255, 255)
        } else {
            Rgba::from_channels((cr / count) as u8,
                                (cg / count) as u8,
                                (cb / count) as u8,
                                (ca / count) as u8)
        }
    }

    fn generate_primitive(&mut self) -> Triangle {
        Triangle::new(self.point_gen.next_point(),
                      self.point_gen.next_point(),
                      self.point_gen.next_point())
    }

    fn add_to_image(image: &mut RgbaImage, primitive: Triangle) {
        let bb = primitive.get_bounding_box();

        for y in bb.top_left.y..bb.bottom_right.y {
            for x in bb.top_left.x..bb.bottom_right.x {
        // for y in 0..image.height() {
        //     for x in 0..image.width() {
                let p = Point {x: x as i32, y: y as i32};
                if primitive.is_inside_primitive(p) {
                    *image.get_pixel_mut(x as u32, y as u32) = Geometrify::mixColor(primitive.color.expect("color of triangle not set"), *image.get_pixel(x as u32, y as u32));
                }
            }
        }
    }

    fn mixColor(first: Rgba<u8>, second: Rgba<u8>) -> Rgba<u8> {
        let (r1, g1, b1, a1) = first.channels4();
        let (r2, g2, b2, a2) = second.channels4();

        Rgba::from_channels(
            ((((r1 as u32 + r2 as u32)) / 2) as u8),
            ((((g1 as u32 + g2 as u32)) / 2) as u8),
            ((((b1 as u32 + b2 as u32)) / 2) as u8),
            ((((a1 as u32 + a2 as u32)) / 2) as u8),
        )
    }

    fn difference(first: Rgba<u8>, second: Rgba<u8>) -> i32 {
        let (r1, g1, b1, a1) = first.channels4();
        let (r2, g2, b2, a2) = second.channels4();
        let mut d = 0i32;

        d += i32::abs((r1 as i32 - r2 as i32));
        d += i32::abs((g1 as i32 - g2 as i32));
        d += i32::abs((b1 as i32 - b2 as i32));
        d += i32::abs((a1 as i32 - a2 as i32));

        d
    }

    fn calculate_difference(original: &RgbaImage, current: &RgbaImage, primitive: Triangle) -> i32 {
        let bb = primitive.get_bounding_box();

        let mut d = 0i32;

        for y in 0..original.height() {
            for x in 0..original.width() {
                let originalRgb = original.get_pixel(x, y);
                let resultRgb = if (bb.top_left.x as u32 <= x) && (x <= bb.bottom_right.x as u32)
                    && (bb.top_left.y as u32 <= y) && (y <= bb.bottom_right.y as u32)
                    && (primitive.is_inside_primitive(Point { x: x as i32, y: y as i32 })) {
                        Geometrify::mixColor(*current.get_pixel(x, y), primitive.color.expect("triangle color not "))
                    } else {
                        *current.get_pixel(x, y)
                    };

                d += Geometrify::difference(*originalRgb, resultRgb);
            }
        }

        d
    }

    pub fn apply(&mut self, image: RgbaImage, numberOfIterations: i32, numberOfSamples: i32) -> RgbaImage {
        let mut destination = RgbaImage::new(image.width(), image.height());

        for _ in 0..numberOfIterations {
            let minPrimitive = (0..numberOfSamples).map(|_| {
                self.generate_primitive()
            }).collect::<Vec<Triangle>>().par_iter()
                .map(|primitive| {
                    let mut prim = *primitive;
                    prim.color = Some(Geometrify::calculate_color(&image, prim));
                    (prim, Geometrify::calculate_difference(&image, &destination, prim))
                }).min_by_key(|tup| tup.1);

            Geometrify::add_to_image(&mut destination, minPrimitive.expect("no fitting triangle found").0);
        }

        destination
    }

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
