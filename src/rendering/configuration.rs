pub struct RenderingConfiguration {
    #[allow(unused)]
    pub msaa: bool,
    pub max_width: usize,
    pub max_height: usize,
    pub width: usize,
    pub height: usize,
}

impl RenderingConfiguration {
    pub fn max_pixels(&self) -> usize {
        self.max_width * self.max_height
    }

    pub fn pixels(&self) -> usize {
        self.width * self.height
    }
}
