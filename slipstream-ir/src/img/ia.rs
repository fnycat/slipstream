use byteorder::ReadBytesExt;
use glam::Vec2Swizzles;
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::img::{EncodedTexture, Rgba8Texture, block_count, detile, hash_pixels};
use crate::tex0::TextureFormat;

/// Single channel 4-bit intensity values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ia4Texture {
    pub(super) dimensions: glam::U16Vec2,
    pub(super) mipmaps: u32,
    /// Intensity values. The values have already been multiplied by `0x11` compared
    /// to the disk format.
    pub(super) pixels: Vec<glam::U8Vec2>,
    pub(super) hash: u64,
}

impl Ia4Texture {
    /// The format of this texture when uploaded to the GPU.
    #[cfg(feature = "egui-impls")]
    pub const SHADER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg8Unorm;

    pub const BLOCK_W: usize = 8;
    pub const BLOCK_H: usize = 4;
    pub const BLOCK_SIZE: glam::USizeVec2 = glam::usizevec2(Self::BLOCK_W, Self::BLOCK_H);

    pub fn new(
        reader: &mut RefCursor<[u8]>,
        dimensions: glam::U16Vec2,
        mipmaps: u32,
    ) -> SlipstreamResult<Self> {
        let blocks = block_count(dimensions, Self::BLOCK_SIZE);

        let mut tiles = Vec::with_capacity(dimensions.x as usize * dimensions.y as usize);
        for _ in 0..blocks {
            for _ in 0..(Self::BLOCK_W * Self::BLOCK_H / 2) {
                let byte = reader.read_u8()?;
                let a = (byte & 0x0F) * 0x11;
                let rgb = ((byte >> 4) & 0x0F) * 0x11;

                tiles.push(glam::u8vec2(rgb, a))
            }
        }

        let pixels = detile::<8, 4, 32, _>(&tiles, dimensions);
        let hash = hash_pixels(bytemuck::cast_slice(&pixels));

        Ok(Self {
            dimensions,
            mipmaps,
            pixels,
            hash,
        })
    }
}

impl EncodedTexture for Ia4Texture {
    fn format(&self) -> TextureFormat {
        TextureFormat::I4
    }

    fn dimensions(&self) -> glam::U16Vec2 {
        self.dimensions
    }

    fn data_len(&self) -> usize {
        todo!()
    }

    fn to_rgba8_unorm(&self) -> SlipstreamResult<Rgba8Texture> {
        let out = self
            .pixels
            .iter()
            .copied()
            .map(|i| i.xxxy())
            .collect::<Vec<_>>();

        Ok(Rgba8Texture::from(self))
    }

    fn mipmaps(&self) -> u32 {
        self.mipmaps
    }

    fn content_hash(&self) -> u64 {
        self.hash
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ia8Texture {
    pub(super) dimensions: glam::U16Vec2,
    pub(super) mipmaps: u32,
    pub(super) pixels: Vec<glam::U8Vec2>,
    pub(super) hash: u64,
}

impl Ia8Texture {
    /// The format of this texture when uploaded to the GPU.
    #[cfg(feature = "egui-impls")]
    pub const SHADER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg8Unorm;

    pub const BLOCK_W: usize = 4;
    pub const BLOCK_H: usize = 4;
    pub const BLOCK_SIZE: glam::USizeVec2 = glam::usizevec2(Self::BLOCK_W, Self::BLOCK_H);

    pub fn new(
        reader: &mut RefCursor<[u8]>,
        dimensions: glam::U16Vec2,
        mipmaps: u32,
    ) -> SlipstreamResult<Self> {
        let blocks = block_count(dimensions, Self::BLOCK_SIZE);

        let mut tiled = Vec::with_capacity(dimensions.x as usize * dimensions.y as usize);
        for _ in 0..blocks {
            for _ in 0..(Self::BLOCK_W * Self::BLOCK_H) {
                let a = reader.read_u8()?;
                let rgb = reader.read_u8()?;

                tiled.push(glam::u8vec2(rgb, a));
            }
        }

        let pixels = detile::<4, 4, 16, _>(&tiled, dimensions);
        let hash = hash_pixels(bytemuck::cast_slice(&pixels));

        Ok(Self {
            dimensions,
            mipmaps,
            pixels,
            hash,
        })
    }
}

impl EncodedTexture for Ia8Texture {
    fn format(&self) -> TextureFormat {
        TextureFormat::I8
    }

    fn dimensions(&self) -> glam::U16Vec2 {
        self.dimensions
    }

    fn data_len(&self) -> usize {
        todo!()
    }

    fn to_rgba8_unorm(&self) -> SlipstreamResult<Rgba8Texture> {
        Ok(Rgba8Texture::from(self))
    }

    fn mipmaps(&self) -> u32 {
        self.mipmaps
    }

    fn content_hash(&self) -> u64 {
        self.hash
    }
}
