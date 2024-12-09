use bytemuck::{Pod, Zeroable};
use getset::Getters;

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod, Getters, Default, Debug)]
pub struct Interval {
    #[getset(get = "pub")]
    min: f32,
    #[getset(get = "pub")]
    max: f32,
    _padding: [u32; 2],
}

impl Interval {
    pub const fn empty() -> Self {
        const EMPTY: Interval = Interval {
            min: f32::MAX,
            max: f32::MIN,
            _padding: [0; 2],
        };

        EMPTY
    }

    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            _padding: [0; 2],
        }
    }

    pub fn new_from_intervals(a: &Interval, b: &Interval) -> Self {
        Self::new(a.min.min(b.min), a.max.max(b.max))
    }

    pub fn merge(&mut self, other: &Interval) {
        if other.min < self.min {
            self.min = other.min;
        }
        if other.max > self.max {
            self.max = other.max;
        }
    }

    pub fn size(&self) -> f32 {
        self.max - self.min
    }

    pub fn contains(&self, x: f32) -> bool {
        self.min <= x && x <= self.max
    }

    pub fn surrounds(&self, x: f32) -> bool {
        self.min < x && x < self.max
    }

    pub fn clamp(&self, x: f32) -> f32 {
        if x < self.min {
            self.min
        } else if x > self.max {
            self.max
        } else {
            x
        }
    }

    pub fn expand(&mut self, delta: f32) -> &mut Self {
        let padding = delta / 2.0;
        self.min -= padding;
        self.max += padding;
        self
    }

    pub fn displace(&mut self, displacement: f32) -> &mut Self {
        self.min += displacement;
        self.max += displacement;
        self
    }
}
