/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use super::{Point, BoundingBox, PointGenerator, Filter};
use image::{Rgba, RgbaImage, Pixel};

use pbr::ProgressBar;

use rayon::prelude::*;

trait Primitive {
    fn is_inside_primitive(&self, p: Point) -> bool;
    fn bounding_box(&self) -> BoundingBox;
    fn get_color(&self) -> Option<Rgba<u8>>;
    fn set_color(&mut self, color: Rgba<u8>);
}

#[derive(Debug, Copy, Clone)]
struct Triangle {
    a: Point,
    b: Point,
    c: Point,
    color: Option<Rgba<u8>>,

    span_div: Option<f32>,
}

impl Triangle {
    fn new(a: Point, b: Point, c: Point) -> Triangle {
        Triangle {
            a: a,
            b: b,
            c: c,
            color: None,
            span_div: None,
        }
    }

    fn span_div_save(&mut self) {
        self.span_div = Some(self.span_div());
    }

    fn span_div(&self) -> f32 {
        match self.span_div {
            Some(div) => div,
            None => {
                let span_a = Point {
                    x: self.b.x - self.a.x,
                    y: self.b.y - self.a.y,
                };
                let span_b = Point {
                    x: self.c.x - self.a.x,
                    y: self.c.y - self.a.y,
                };

                1.0 / span_a.cross_product(span_b) as f32
            }
        }
    }
}

impl Primitive for Triangle {
    fn is_inside_primitive(&self, p: Point) -> bool {
        let span_a = Point {
            x: self.b.x - self.a.x,
            y: self.b.y - self.a.y,
        };
        let span_b = Point {
            x: self.c.x - self.a.x,
            y: self.c.y - self.a.y,
        };

        let q = Point {
            x: p.x - self.a.x,
            y: p.y - self.a.y,
        };

        let s = q.cross_product(span_b) as f32 * self.span_div();
        let t = span_a.cross_product(q) as f32 * self.span_div();

        (s >= 0.0) && (t >= 0.0) && ((s + t) <= 1.0)
    }

    fn bounding_box(&self) -> BoundingBox {
        use std::cmp::{min, max};
        BoundingBox {
            top_left: Point {
                x: min(min(self.a.x, self.b.x), self.c.x),
                y: min(min(self.a.y, self.b.y), self.c.y),
            },
            bottom_right: Point {
                x: max(max(self.a.x, self.b.x), self.c.x),
                y: max(max(self.a.y, self.b.y), self.c.y),
            },
        }
    }

    fn get_color(&self) -> Option<Rgba<u8>> {
        self.color
    }

    fn set_color(&mut self, color: Rgba<u8>) {
        self.color = Some(color);
    }
}

pub struct Geometrify {
    point_gen: Box<PointGenerator>,
    iterations: u32,
    samples: u32,
}

impl Geometrify {
    pub fn new(point_gen: Box<PointGenerator>, iterations: u32, samples: u32) -> Geometrify {
        Geometrify {
            point_gen: point_gen,
            iterations: iterations,
            samples: samples,
        }
    }

    pub fn set_iterations(&mut self, iterations: u32) {
        self.iterations = iterations
    }

    pub fn set_samples(&mut self, samples: u32) {
        self.samples = samples
    }

    fn calculate_color(image: &RgbaImage, primitive: &Primitive) -> Rgba<u8> {
        let bb = primitive.bounding_box();

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
            Rgba::from_channels(
                (cr / count) as u8,
                (cg / count) as u8,
                (cb / count) as u8,
                (ca / count) as u8,
            )
        }
    }

    fn generate_primitive(&mut self, width: u32, height: u32) -> Triangle {
        Triangle::new(
            self.point_gen.next_point(width, height),
            self.point_gen.next_point(width, height),
            self.point_gen.next_point(width, height),
        )
    }

    fn add_to_image(image: &mut RgbaImage, primitive: &Primitive) {
        let bb = primitive.bounding_box();

        for y in bb.top_left.y..bb.bottom_right.y {
            for x in bb.top_left.x..bb.bottom_right.x {
                let p = Point {
                    x: x as i32,
                    y: y as i32,
                };
                if primitive.is_inside_primitive(p) {
                    *image.get_pixel_mut(x as u32, y as u32) =
                        Geometrify::mix_color(
                            primitive.get_color().expect("color of triangle not set"),
                            *image.get_pixel(x as u32, y as u32),
                        );
                }
            }
        }
    }

    fn mix_color(first: Rgba<u8>, second: Rgba<u8>) -> Rgba<u8> {
        let (r1, g1, b1, a1) = first.channels4();
        let (r2, g2, b2, a2) = second.channels4();

        Rgba::from_channels(
            (((r1 as u32 + r2 as u32) / 2) as u8),
            (((g1 as u32 + g2 as u32) / 2) as u8),
            (((b1 as u32 + b2 as u32) / 2) as u8),
            (((a1 as u32 + a2 as u32) / 2) as u8),
        )
    }

    fn difference(first: Rgba<u8>, second: Rgba<u8>) -> u32 {
        let (r1, g1, b1, a1) = first.channels4();
        let (r2, g2, b2, a2) = second.channels4();
        let mut d = 0i32;

        d += i32::abs(r1 as i32 - r2 as i32);
        d += i32::abs(g1 as i32 - g2 as i32);
        d += i32::abs(b1 as i32 - b2 as i32);
        d += i32::abs(a1 as i32 - a2 as i32);

        d as u32
    }

    fn calculate_difference(original: &RgbaImage,
                            current: &RgbaImage,
                            diff_lut: &[u64],
                            primitive: &Primitive)
                            -> u64 {
        let bb = primitive.bounding_box();

        // Use LUT to calculate difference outside of the BB
        // TODO: Check whether indices are correct!
        let mut d = diff_lut[diff_lut.len() - 1];
        if bb.bottom_right.y > 0 && bb.bottom_right.x > 0 {
            d -= diff_lut[((bb.bottom_right.y - 1) as u32 * current.width() +
             bb.bottom_right.x as u32 - 1) as usize];
        }
        if bb.top_left.y > 0 && bb.bottom_right.x > 0 {
            d += diff_lut[((bb.top_left.y - 1) as u32 * current.width() +
             bb.bottom_right.x as u32 - 1) as usize];
        }
        if bb.bottom_right.y > 0 && bb.top_left.x > 0 {
            d += diff_lut[((bb.bottom_right.y - 1) as u32 * current.width() +
             bb.top_left.x as u32 - 1) as usize];
        }
        if bb.top_left.y > 0 && bb.top_left.x > 0 {
            d -= diff_lut[((bb.top_left.y - 1) as u32 * current.width() +
             bb.top_left.x as u32 - 1) as usize];
        }

        for y in bb.top_left.y..bb.bottom_right.y {
            for x in bb.top_left.x..bb.bottom_right.x {
                let original_rgb = original.get_pixel(x as u32, y as u32);
                let result_rgb = if (bb.top_left.x as u32 <= x as u32) &&
                                    (x as u32 <= bb.bottom_right.x as u32) &&
                                    (bb.top_left.y as u32 <= y as u32) &&
                                    (y as u32 <= bb.bottom_right.y as u32) &&
                                    (primitive.is_inside_primitive(
                    Point {
                        x: x as i32,
                        y: y as i32,
                    }
                )) {
                    Geometrify::mix_color(
                        *current.get_pixel(x as u32, y as u32),
                        primitive.get_color().expect("triangle color not "),
                    )
                } else {
                    *current.get_pixel(x as u32, y as u32)
                };

                d += Geometrify::difference(*original_rgb, result_rgb) as u64;
            }
        }

        d
    }

    fn calculate_difference_lut(a: &RgbaImage, b: &RgbaImage) -> Vec<u64> {
        let mut result = Vec::new();

        for y in 0..a.height() {
            for x in 0..a.width() {
                let mut ldiff = Geometrify::difference(*a.get_pixel(x, y), *b.get_pixel(x, y)) as
                                u64;
                if x > 0 {
                    ldiff += result[(y * a.width() + x - 1) as usize];
                }
                if y > 0 {
                    ldiff += result[((y - 1) * a.width() + x) as usize];
                }
                if x > 0 && y > 0 {
                    ldiff -= result[((y - 1) * a.width() + x - 1) as usize];
                }

                result.push(ldiff);
            }
        }

        result
    }
}

impl Filter for Geometrify {
    fn apply(&mut self, image: &RgbaImage) -> RgbaImage {
        let mut progress = ProgressBar::new(self.iterations as u64);
        progress.format("|#--|");

        let mut destination = RgbaImage::new(image.width(), image.height());

        for _ in 0..self.iterations {
            let difference_lut = Geometrify::calculate_difference_lut(&image, &destination);

            let primitives = (0..self.samples)
                .map(|_| self.generate_primitive(image.width(), image.height()))
                .map(
                    |mut p| {
                        p.span_div_save();
                        p
                    }
                )
                .collect::<Vec<Triangle>>();
            let min_primitive = primitives
                .par_iter()
                .map(
                    |primitive| {
                        let mut prim = *primitive;
                        prim.color = Some(Geometrify::calculate_color(&image, &prim));
                        (prim,
                         Geometrify::calculate_difference(
                            &image,
                            &destination,
                            &difference_lut,
                            &prim,
                        ))
                    }
                )
                .min_by_key(|tup| tup.1);

            Geometrify::add_to_image(
                &mut destination,
                &min_primitive.expect("no fitting triangle found").0,
            );
            progress.inc();
        }
        progress.finish_print("done");

        destination
    }
}
