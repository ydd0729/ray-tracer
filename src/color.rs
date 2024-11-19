pub(crate) struct StaticColor {}

impl StaticColor {
    pub(crate) const CLEAR: wgpu::Color = wgpu::Color {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 1.0,
    };
}