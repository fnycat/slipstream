#[derive(Clone, PartialEq, Eq)]
pub struct Rgba8UnormTexture {
    size: glam::U16Vec2,
    pixels: Vec<glam::U8Vec4>,
}

impl Rgba8UnormTexture {
    pub fn new(size: glam::U16Vec2, pixels: Vec<glam::U8Vec4>) -> Self {
        Self { size, pixels }
    }

    pub fn size(&self) -> glam::U16Vec2 {
        self.size
    }

    pub fn pixels(&self) -> &[glam::U8Vec4] {
        &self.pixels
    }
}
