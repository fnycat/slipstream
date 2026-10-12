use slipstream_shared::SlipstreamResult;

use crate::{
    img::{
        EncodedTexture, I4Texture, I8Texture, Ia4Texture, Ia8Texture, Rgb5a3Texture, Rgb565Texture,
    },
    tex0::TextureFormat,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Rgba8Texture {
    pub(super) dimensions: glam::U16Vec2,
    pub(super) pixels: Vec<glam::U8Vec4>,
    pub(super) mipmaps: u32,
    pub(super) hash: u64,
}

impl Rgba8Texture {
    pub fn bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.pixels)
    }
}

impl EncodedTexture for Rgba8Texture {
    fn format(&self) -> TextureFormat {
        TextureFormat::Rgba8
    }

    fn dimensions(&self) -> glam::U16Vec2 {
        self.dimensions
    }

    fn data_len(&self) -> usize {
        todo!()
    }

    fn to_rgba8_unorm(&self) -> SlipstreamResult<Rgba8Texture> {
        Ok(self.clone())
    }

    fn mipmaps(&self) -> u32 {
        self.mipmaps
    }

    fn content_hash(&self) -> u64 {
        self.hash
    }
}

impl From<&I4Texture> for Rgba8Texture {
    fn from(value: &I4Texture) -> Self {
        Self {
            dimensions: value.dimensions,
            mipmaps: value.mipmaps,
            hash: value.hash,
            pixels: value
                .pixels
                .iter()
                .map(|&p| glam::u8vec4(p, p, p, 0xff))
                .collect(),
        }
    }
}

impl From<&I8Texture> for Rgba8Texture {
    fn from(value: &I8Texture) -> Self {
        Self {
            dimensions: value.dimensions,
            mipmaps: value.mipmaps,
            hash: value.hash,
            pixels: value
                .pixels
                .iter()
                .map(|&p| glam::u8vec4(p, p, p, 0xff))
                .collect(),
        }
    }
}

impl From<&Ia4Texture> for Rgba8Texture {
    fn from(value: &Ia4Texture) -> Self {
        Self {
            dimensions: value.dimensions,
            mipmaps: value.mipmaps,
            hash: value.hash,
            pixels: value
                .pixels
                .iter()
                .map(|&p| glam::u8vec4(p.x, p.x, p.x, p.y))
                .collect(),
        }
    }
}

impl From<&Ia8Texture> for Rgba8Texture {
    fn from(value: &Ia8Texture) -> Self {
        Self {
            dimensions: value.dimensions,
            mipmaps: value.mipmaps,
            hash: value.hash,
            pixels: value
                .pixels
                .iter()
                .map(|&p| glam::u8vec4(p.x, p.x, p.x, p.y))
                .collect(),
        }
    }
}

impl From<&Rgb5a3Texture> for Rgba8Texture {
    fn from(value: &Rgb5a3Texture) -> Self {
        Self {
            dimensions: value.dimensions,
            mipmaps: value.mipmaps,
            hash: value.hash,
            pixels: value.pixels.clone(),
        }
    }
}

impl From<&Rgb565Texture> for Rgba8Texture {
    fn from(value: &Rgb565Texture) -> Self {
        Self {
            dimensions: value.dimensions,
            mipmaps: value.mipmaps,
            hash: value.hash,
            pixels: value.pixels.clone(),
        }
    }
}
