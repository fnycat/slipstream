use byteorder::ReadBytesExt;
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::img::{
    EncodedTexture, I4Texture, I8Texture, Ia4Texture, Ia8Texture, Rgb5a3Texture, Rgb565Texture,
    block_count, detile, hash_pixels
};
use crate::tex0::TextureFormat;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgba8Texture {
    pub(super) dimensions: glam::U16Vec2,
    pub(super) pixels: Vec<glam::U8Vec4>,
    pub(super) mipmaps: u32,
    pub(super) hash: u64
}

impl Rgba8Texture {
    #[cfg(feature = "egui-impls")]
    pub const SHADER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    pub const BLOCK_W: usize = 4;
    pub const BLOCK_H: usize = 4;
    pub const BLOCK_SIZE: glam::USizeVec2 = glam::usizevec2(Self::BLOCK_W, Self::BLOCK_H);

    pub fn new(
        reader: &mut RefCursor<[u8]>,
        dimensions: glam::U16Vec2,
        mipmaps: u32
    ) -> SlipstreamResult<Self> {
        let blocks = block_count(dimensions, Self::BLOCK_SIZE);

        let mut tiled = Vec::with_capacity(dimensions.x as usize * dimensions.y as usize);
        let mut curr_block = [glam::U8Vec4::ZERO; 16];
        for _ in 0..blocks {
            // Decode alpha and red channels.
            for i in 0..(Self::BLOCK_W * Self::BLOCK_H) {
                curr_block[i].w = reader.read_u8()?;
                curr_block[i].x = reader.read_u8()?;
            }

            // Decode blue and green channels
            for i in 0..(Self::BLOCK_W * Self::BLOCK_H) {
                curr_block[i].y = reader.read_u8()?;
                curr_block[i].z = reader.read_u8()?;
            }

            tiled.extend_from_slice(&curr_block);
        }

        let pixels = detile::<4, 4, 16, _>(&tiled, dimensions);
        let hash = hash_pixels(bytemuck::cast_slice(&pixels));

        Ok(Self {
            dimensions,
            pixels,
            mipmaps,
            hash
        })
    }

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
                .collect()
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
                .collect()
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
                .collect()
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
                .collect()
        }
    }
}

impl From<&Rgb5a3Texture> for Rgba8Texture {
    fn from(value: &Rgb5a3Texture) -> Self {
        Self {
            dimensions: value.dimensions,
            mipmaps: value.mipmaps,
            hash: value.hash,
            pixels: value.pixels.clone()
        }
    }
}

impl From<&Rgb565Texture> for Rgba8Texture {
    fn from(value: &Rgb565Texture) -> Self {
        Self {
            dimensions: value.dimensions,
            mipmaps: value.mipmaps,
            hash: value.hash,
            pixels: value.pixels.clone()
        }
    }
}
