/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

pub mod geometrify;

use image::RgbaImage;
use nanorand::{Rng, WyRand};

#[derive(Clone, Copy, Debug)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn cross_product(&self, b: Point) -> i32 {
        self.x * b.y - self.y * b.x
    }
}

#[derive(Debug, Copy, Clone)]
struct BoundingBox {
    top_left: Point,
    bottom_right: Point,
}

pub trait PointGenerator {
    fn next_point(&mut self, width: u32, height: u32) -> Point;
}

pub struct RandomPointGenerator {
    rng: WyRand,
}

impl RandomPointGenerator {
    pub fn new() -> RandomPointGenerator {
        RandomPointGenerator { rng: WyRand::new() }
    }
}

impl Default for RandomPointGenerator {
    fn default() -> Self {
        RandomPointGenerator::new()
    }
}

impl PointGenerator for RandomPointGenerator {
    fn next_point(&mut self, width: u32, height: u32) -> Point {
        // nanorand seems to be broken as of 2022-05-23... When the range is specified in i32, then
        // it will return -1 even if the lower bound is zero... WTF
        Point {
            x: self.rng.generate_range(0..width) as i32,
            y: self.rng.generate_range(0..height) as i32,
        }
    }
}

pub trait ProgressReporter {
    fn init(&mut self, count: u64);
    fn step(&mut self);
    fn finish(&mut self);
}

pub struct SilentProgressReporter;

impl SilentProgressReporter {
    pub fn new() -> SilentProgressReporter {
        SilentProgressReporter {}
    }
}

impl Default for SilentProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressReporter for SilentProgressReporter {
    fn init(&mut self, _: u64) {}
    fn step(&mut self) {}
    fn finish(&mut self) {}
}

pub trait Filter {
    fn apply(&mut self, image: &RgbaImage, progress: &mut dyn ProgressReporter) -> RgbaImage;
}
