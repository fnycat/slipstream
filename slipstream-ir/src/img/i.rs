use std::hash::{DefaultHasher, Hash, Hasher};

use byteorder::ReadBytesExt;

use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::{
    img::{EncodedTexture, Rgba8UnormTexture, detile, hash_pixels},
    tex0::TextureFormat,
};

/// Single channel 4-bit intensity values.
#[derive(Debug, Clone, PartialEq)]
pub struct I4Texture {
    pub dimensions: glam::U16Vec2,
    pub mipmaps: u32,
    /// Intensity values. The values have already been multiplied by `0x11` compared
    /// to the disk format.
    pub pixels: Vec<u8>,

    /// A hash of the pixel's contents.
    pub hash: u64,
}

impl I4Texture {
    /// The format of this texture when uploaded to the GPU.
    #[cfg(feature = "egui-impls")]
    pub const SHADER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

    pub const PAIRS_PER_BLOCK: usize = 8 * 8 / 2;

    pub fn new(
        reader: &mut RefCursor<[u8]>,
        dimensions: glam::U16Vec2,
        mipmaps: u32,
    ) -> SlipstreamResult<Self> {
        let blocks = {
            let blocks_x = dimensions.x.div_ceil(8);
            let blocks_y = dimensions.y.div_ceil(8);

            blocks_x as usize * blocks_y as usize
        };

        // Decode into tiles.
        let mut tiles = Vec::with_capacity(dimensions.x as usize * dimensions.y as usize);
        for _ in 0..blocks {
            // Decode an 8x8 pixel block.
            for _ in 0..Self::PAIRS_PER_BLOCK {
                let pair = reader.read_u8()?;

                // multiply by 0x11 to map from 4 bit to 8 bit range.
                tiles.push((pair & 0x0F) * 0x11);
                tiles.push(((pair >> 4) & 0x0F) * 0x11);
            }
        }

        // Then remove the tile pattern
        let pixels = detile::<8, 8, 64>(&tiles, dimensions);
        let hash = hash_pixels(&pixels);

        Ok(Self {
            dimensions,
            mipmaps,
            pixels,
            hash,
        })
    }
}

impl EncodedTexture for I4Texture {
    fn format(&self) -> TextureFormat {
        TextureFormat::I4
    }

    fn dimensions(&self) -> glam::U16Vec2 {
        self.dimensions
    }

    fn data_len(&self) -> usize {
        todo!()
    }

    fn to_rgba8_unorm(&self) -> SlipstreamResult<Rgba8UnormTexture> {
        let out = self
            .pixels
            .iter()
            .copied()
            .map(|i| glam::u8vec4(i, i, i, 0xff))
            .collect::<Vec<_>>();

        Ok(Rgba8UnormTexture::new(self.dimensions, out))
    }

    fn mipmaps(&self) -> u32 {
        self.mipmaps
    }

    fn content_hash(&self) -> u64 {
        self.hash
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct I8Texture {
    dimensions: glam::U16Vec2,
    mipmaps: u32,
    pixels: Vec<u8>,
    hash: u64,
}

impl I8Texture {
    /// The format of this texture when uploaded to the GPU.
    #[cfg(feature = "egui-impls")]
    pub const SHADER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

    pub const PIXELS_PER_BLOCK: usize = 8 * 8;

    pub fn new(
        reader: &mut RefCursor<[u8]>,
        dimensions: glam::U16Vec2,
        mipmaps: u32,
    ) -> SlipstreamResult<Self> {
        let blocks = {
            let blocks_x = dimensions.x.div_ceil(8);
            let blocks_y = dimensions.y.div_ceil(8);

            blocks_x as usize * blocks_y as usize
        };

        let mut tiles = Vec::with_capacity(dimensions.x as usize * dimensions.y as usize);
        for _ in 0..blocks {
            // Decode an 8x8 pixel block.
            for _ in 0..Self::PIXELS_PER_BLOCK {
                tiles.push(reader.read_u8()?);
            }
        }

        let pixels = detile::<8, 4, 32>(&tiles, dimensions);
        let hash = hash_pixels(&pixels);

        Ok(Self {
            dimensions,
            mipmaps,
            pixels,
            hash,
        })
    }
}

impl EncodedTexture for I8Texture {
    fn format(&self) -> TextureFormat {
        TextureFormat::I8
    }

    fn dimensions(&self) -> glam::U16Vec2 {
        self.dimensions
    }

    fn data_len(&self) -> usize {
        todo!()
    }

    fn to_rgba8_unorm(&self) -> SlipstreamResult<Rgba8UnormTexture> {
        let out = self
            .pixels
            .iter()
            .copied()
            .map(|i| glam::u8vec4(i, i, i, 0xff))
            .collect::<Vec<_>>();

        Ok(Rgba8UnormTexture::new(self.dimensions, out))
    }

    fn mipmaps(&self) -> u32 {
        self.mipmaps
    }

    fn content_hash(&self) -> u64 {
        self.hash
    }
}
