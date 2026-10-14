use std::io::{Seek, Write};

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::img::{
    EncodedTexture, ExportableTexture, Rgba8Texture, block_count, detile, hash_pixels
};
use crate::tex0::TextureFormat;

/// Converts a 16-byte integer representing an RGB565 colour to an RGBA8 colour.
///
/// This is a separate function, as the CMPR format requires both the short and colour value.
pub const fn from_rgb565(short: u16) -> glam::U8Vec4 {
    let r = ((short & 0xf8_00) >> 11) as u8; // Take 5 bits
    let g = ((short & 0x07_e0) >> 5) as u8; // Then another 6 bits
    let b = (short & 0x00_1f) as u8; // and lastly another 5 bits

    // rescale the components to the full 0-255 range.
    let r = (r << 3) | (r >> 2);
    let g = (g << 2) | (g >> 4);
    let b = (b << 3) | (b >> 2);

    glam::u8vec4(r, g, b, 255)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rgb565Texture {
    pub(super) dimensions: glam::U16Vec2,
    pub(super) mipmaps: u32,
    pub(super) hash: u64,
    pub(super) pixels: Vec<glam::U8Vec4>
}

impl Rgb565Texture {
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
        for _ in 0..blocks {
            // Decode an 8x8 pixel block.
            for _ in 0..(Self::BLOCK_W * Self::BLOCK_H) {
                let color = from_rgb565(reader.read_u16::<BigEndian>()?);
                tiled.push(color);
            }
        }

        let pixels = detile::<4, 4, 16, _>(&tiled, dimensions);
        let hash = hash_pixels(bytemuck::cast_slice(&pixels));

        Ok(Self {
            dimensions,
            mipmaps,
            pixels,
            hash
        })
    }
}

impl EncodedTexture for Rgb565Texture {
    fn format(&self) -> TextureFormat {
        TextureFormat::Rgb565
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

impl ExportableTexture for Rgb565Texture {
    fn export<W: Write + Seek>(
        &self,
        buffered_writer: &mut W,
        format: image::ImageFormat
    ) -> SlipstreamResult<()> {
        let dims = self.dimensions();

        // The image is already converted to RGBA8 format when reading, so we can write it to an image immediately.
        image::write_buffer_with_format(
            buffered_writer,
            self.bytes()
                .expect("Rgb565Texture::bytes should be infallible"),
            dims.x as u32,
            dims.y as u32,
            image::ExtendedColorType::Rgba8,
            format
        )?;

        Ok(())
    }
}
