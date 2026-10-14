use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{Seek, Write};

use byteorder::ReadBytesExt;

use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::img::{
    EncodedTexture, ExportableTexture, Rgba8Texture, block_count, detile, hash_pixels
};
use crate::tex0::TextureFormat;

/// Single channel 4-bit intensity values.
#[derive(Debug, Clone, PartialEq)]
pub struct L4Texture {
    pub dimensions: glam::U16Vec2,
    pub mipmaps: u32,
    /// Intensity values. The values have already been multiplied by `0x11` compared
    /// to the disk format.
    pub pixels: Vec<u8>,

    /// A hash of the pixel's contents.
    pub hash: u64
}

impl L4Texture {
    /// The format of this texture when uploaded to the GPU.
    #[cfg(feature = "egui-impls")]
    pub const SHADER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

    pub const BLOCK_W: usize = 8;
    pub const BLOCK_H: usize = 8;
    pub const BLOCK_SIZE: glam::USizeVec2 = glam::usizevec2(Self::BLOCK_W, Self::BLOCK_H);

    pub fn new(
        reader: &mut RefCursor<[u8]>,
        dimensions: glam::U16Vec2,
        mipmaps: u32
    ) -> SlipstreamResult<Self> {
        let blocks = block_count(dimensions, Self::BLOCK_SIZE);

        // Decode into tiles.
        let mut tiles = Vec::with_capacity(dimensions.x as usize * dimensions.y as usize);
        for _ in 0..blocks {
            // Decode an 8x8 pixel block.
            for _ in 0..(Self::BLOCK_W * Self::BLOCK_H / 2) {
                let pair = reader.read_u8()?;

                // multiply by 0x11 to map from 4 bit to 8 bit range.
                tiles.push((pair & 0x0F) * 0x11);
                tiles.push(((pair >> 4) & 0x0F) * 0x11);
            }
        }

        // Then remove the tile pattern
        let pixels = detile::<8, 8, 64, _>(&tiles, dimensions);
        let hash = hash_pixels(&pixels);

        Ok(Self {
            dimensions,
            mipmaps,
            pixels,
            hash
        })
    }
}

impl EncodedTexture for L4Texture {
    fn format(&self) -> TextureFormat {
        TextureFormat::Luminance4
    }

    fn dimensions(&self) -> glam::U16Vec2 {
        self.dimensions
    }

    fn bytes(&self) -> Option<&[u8]> {
        Some(bytemuck::cast_slice(&self.pixels))
    }

    fn to_rgba8(&self) -> SlipstreamResult<Rgba8Texture> {
        Ok(Rgba8Texture::from(self))
    }

    fn mipmaps(&self) -> u32 {
        self.mipmaps
    }

    fn content_hash(&self) -> u64 {
        self.hash
    }
}

impl ExportableTexture for L4Texture {
    fn export<W: Write + Seek>(
        &self,
        buffered_writer: &mut W,
        format: image::ImageFormat
    ) -> SlipstreamResult<()> {
        let dims = self.dimensions();

        image::write_buffer_with_format(
            buffered_writer,
            self.bytes().expect("L4Texture::bytes should be infallible"),
            dims.x as u32,
            dims.y as u32,
            image::ExtendedColorType::L4,
            format
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct L8Texture {
    pub(super) dimensions: glam::U16Vec2,
    pub(super) mipmaps: u32,
    pub(super) pixels: Vec<u8>,
    pub(super) hash: u64
}

impl L8Texture {
    /// The format of this texture when uploaded to the GPU.
    #[cfg(feature = "egui-impls")]
    pub const SHADER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

    pub const BLOCK_W: usize = 8;
    pub const BLOCK_H: usize = 4;
    pub const BLOCK_SIZE: glam::USizeVec2 = glam::usizevec2(Self::BLOCK_W, Self::BLOCK_H);

    pub fn new(
        reader: &mut RefCursor<[u8]>,
        dimensions: glam::U16Vec2,
        mipmaps: u32
    ) -> SlipstreamResult<Self> {
        let blocks = block_count(dimensions, Self::BLOCK_SIZE);

        let mut tiles = Vec::with_capacity(dimensions.x as usize * dimensions.y as usize);
        for _ in 0..blocks {
            // Decode an 8x8 pixel block.
            for _ in 0..Self::BLOCK_W * Self::BLOCK_H {
                tiles.push(reader.read_u8()?);
            }
        }

        let pixels = detile::<8, 4, 32, _>(&tiles, dimensions);
        let hash = hash_pixels(&pixels);

        Ok(Self {
            dimensions,
            mipmaps,
            pixels,
            hash
        })
    }
}

impl EncodedTexture for L8Texture {
    fn format(&self) -> TextureFormat {
        TextureFormat::Luminance8
    }

    fn dimensions(&self) -> glam::U16Vec2 {
        self.dimensions
    }

    fn bytes(&self) -> Option<&[u8]> {
        Some(bytemuck::cast_slice(&self.pixels))
    }

    fn to_rgba8(&self) -> SlipstreamResult<Rgba8Texture> {
        Ok(Rgba8Texture::from(self))
    }

    fn mipmaps(&self) -> u32 {
        self.mipmaps
    }

    fn content_hash(&self) -> u64 {
        self.hash
    }
}

impl ExportableTexture for L8Texture {
    fn export<W: Write + Seek>(
        &self,
        buffered_writer: &mut W,
        format: image::ImageFormat
    ) -> SlipstreamResult<()> {
        let dims = self.dimensions();

        image::write_buffer_with_format(
            buffered_writer,
            self.bytes().expect("L8Texture::bytes should be infallible"),
            dims.x as u32,
            dims.y as u32,
            image::ExtendedColorType::L8,
            format
        )?;

        Ok(())
    }
}
