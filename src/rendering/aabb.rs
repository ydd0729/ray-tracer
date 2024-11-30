use crate::rendering::interval::Interval;
use nalgebra::Point3;

#[derive(Copy, Clone)]
pub struct AxisAlignedBoundingBox {
    xyz: [Interval; 3],
}

impl AxisAlignedBoundingBox {
    pub const fn empty() -> Self {
        const EMPTY: AxisAlignedBoundingBox = AxisAlignedBoundingBox {
            xyz: [Interval::empty(); 3],
        };
        EMPTY
    }

    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        let mut bounding_box = Self { xyz: [x, y, z] };
        bounding_box.pad_to_minimums();
        bounding_box
    }

    pub fn new_from_points(a: Point3<f32>, b: Point3<f32>) -> Self {
        let mut bounding_box = Self {
            xyz: [
                if a.x < b.x {
                    Interval::new(a.x, b.x)
                } else {
                    Interval::new(b.x, a.x)
                },
                if a.y < b.y {
                    Interval::new(a.y, b.y)
                } else {
                    Interval::new(b.y, a.y)
                },
                if a.z < b.z {
                    Interval::new(a.z, b.z)
                } else {
                    Interval::new(b.z, a.z)
                },
            ],
        };
        bounding_box.pad_to_minimums();
        bounding_box
    }

    pub fn new_from_boxes(a: &AxisAlignedBoundingBox, b: &AxisAlignedBoundingBox) -> Self {
        Self {
            xyz: [
                Interval::new_from_intervals(a.x(), b.x()),
                Interval::new_from_intervals(a.y(), b.y()),
                Interval::new_from_intervals(a.z(), b.z()),
            ],
        }
    }

    pub fn merge(&mut self, other: &AxisAlignedBoundingBox) {
        for i in 0..3 {
            self.axis_mut(i).merge(&other.axis(i));
        }
    }

    fn pad_to_minimums(&mut self) {
        const DELTA: f32 = 1E-4f32;

        if self.x().size() < DELTA {
            self.x_mut().expand(DELTA);
        }
        if self.y().size() < DELTA {
            self.y_mut().expand(DELTA);
        }
        if self.z().size() < DELTA {
            self.z_mut().expand(DELTA);
        }
    }

    pub fn longest_axis(&self) -> u8 {
        if self.x().size() > self.y().size() {
            if self.x().size() > self.z().size() {
                0
            } else {
                2
            }
        } else if self.y().size() > self.z().size() {
            1
        } else {
            2
        }
    }

    pub fn axis(&self, n: i32) -> &Interval {
        if n == 1 {
            self.y()
        } else if n == 2 {
            self.z()
        } else {
            self.x()
        }
    }

    pub fn axis_mut(&mut self, n: i32) -> &mut Interval {
        if n == 1 {
            self.y_mut()
        } else if n == 2 {
            self.z_mut()
        } else {
            self.x_mut()
        }
    }

    #[inline(always)]
    pub fn x(&self) -> &Interval {
        unsafe { self.xyz.get_unchecked(0) }
    }
    #[inline(always)]
    pub fn y(&self) -> &Interval {
        unsafe { self.xyz.get_unchecked(1) }
    }
    #[inline(always)]
    pub fn z(&self) -> &Interval {
        unsafe { self.xyz.get_unchecked(2) }
    }
    #[inline(always)]
    pub fn x_mut(&mut self) -> &mut Interval {
        unsafe { self.xyz.get_unchecked_mut(0) }
    }
    #[inline(always)]
    pub fn y_mut(&mut self) -> &mut Interval {
        unsafe { self.xyz.get_unchecked_mut(1) }
    }

    #[inline(always)]
    pub fn z_mut(&mut self) -> &mut Interval {
        unsafe { self.xyz.get_unchecked_mut(2) }
    }
}
