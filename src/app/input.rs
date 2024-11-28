use crate::time;
use winit::event::ElementState;

#[derive(Debug)]
pub struct PressRecord {
    pressing: bool,
    pub start: time::Instant,
    pub end: time::Instant,
}

impl PressRecord {
    pub fn press(&mut self) {
        if self.pressing {
            return;
        }

        let now = time::Instant::now();
        self.pressing = true;
        self.start = now;
        self.end = now;
    }

    pub fn release(&mut self) {
        if !self.pressing {
            return;
        }

        let now = time::Instant::now();
        self.pressing = false;
        self.end = now;
    }

    pub fn update(&mut self, state: ElementState) {
        match state {
            ElementState::Pressed => self.press(),
            ElementState::Released => self.release(),
        }
    }

    pub fn delta(&mut self) -> f32 {
        let duration: time::Duration;
        let now = time::Instant::now();

        if self.pressing {
            duration = now - self.start;
            self.start = now;
        } else {
            duration = self.end - self.start;
            self.start = now;
            self.end = now;
        }

        duration.as_secs_f32()
    }
}

impl Default for PressRecord {
    fn default() -> Self {
        let now = time::Instant::now();
        Self {
            pressing: false,
            start: now,
            end: now,
        }
    }
}
