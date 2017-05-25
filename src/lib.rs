extern crate image;
extern crate rand;
extern crate rayon;
extern crate pbr;

pub mod geometrify;

use rand::Rng;

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
    rng: Box<Rng>,
}

impl RandomPointGenerator {
    pub fn new() -> RandomPointGenerator {
        RandomPointGenerator { rng: Box::new(::rand::weak_rng()) }
    }
}

impl Default for RandomPointGenerator {
    fn default() -> Self {
        RandomPointGenerator::new()
    }
}

impl PointGenerator for RandomPointGenerator {
    fn next_point(&mut self, width: u32, height: u32) -> Point {
        Point {
            x: self.rng.gen_range(0, width as i32),
            y: self.rng.gen_range(0, height as i32),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
