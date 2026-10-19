use std::io::{Seek, Write};
use std::ops::Deref;
use std::sync::Arc;

use byteorder::ReadBytesExt;
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::tpl::{
    EncodedImage, ExportableImage, ImageFormat, L4Image, L8Image, La4Image, La8Image, Rgb5a3Image,
    Rgb565Image, block_count, detile, hash_pixels
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgba8Image {
    pub(super) dimensions: glam::U16Vec2,
    pub(super) pixels: Arc<[glam::U8Vec4]>,
    pub(super) mipmaps: u32,
    pub(super) hash: u64
}

impl Rgba8Image {
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

        let pixels = Arc::from(detile::<4, 4, 16, _>(&tiled, dimensions));
        let hash = hash_pixels(bytemuck::cast_slice(&pixels));

        Ok(Self {
            dimensions,
            pixels,
            mipmaps,
            hash
        })
    }

    pub fn bytes(&self) -> Option<&[u8]> {
        Some(bytemuck::cast_slice(&self.pixels))
    }
}

impl EncodedImage for Rgba8Image {
    fn format(&self) -> ImageFormat {
        ImageFormat::Rgba8
    }

    fn dimensions(&self) -> glam::U16Vec2 {
        self.dimensions
    }

    fn bytes(&self) -> Option<&[u8]> {
        Some(bytemuck::cast_slice(&self.pixels))
    }

    fn to_rgba8(&self) -> SlipstreamResult<Rgba8Image> {
        Ok(self.clone())
    }

    fn mipmaps(&self) -> u32 {
        self.mipmaps
    }

    fn content_hash(&self) -> u64 {
        self.hash
    }
}

impl ExportableImage for Rgba8Image {
    #[tracing::instrument(skip_all, fields(format))]
    fn export<W: Write + Seek>(
        &self,
        buffered_writer: &mut W,
        format: image::ImageFormat
    ) -> SlipstreamResult<()> {
        let dims = self.dimensions();

        image::write_buffer_with_format(
            buffered_writer,
            self.bytes()
                .expect("Rgba8Texture::bytes should be infallible"),
            dims.x as u32,
            dims.y as u32,
            image::ExtendedColorType::Rgba8,
            format
        )?;

        Ok(())
    }
}

impl From<&L4Image> for Rgba8Image {
    fn from(value: &L4Image) -> Self {
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

impl From<&L8Image> for Rgba8Image {
    fn from(value: &L8Image) -> Self {
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

impl From<&La4Image> for Rgba8Image {
    fn from(value: &La4Image) -> Self {
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

impl From<&La8Image> for Rgba8Image {
    fn from(value: &La8Image) -> Self {
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

impl From<&Rgb5a3Image> for Rgba8Image {
    fn from(value: &Rgb5a3Image) -> Self {
        Self {
            dimensions: value.dimensions,
            mipmaps: value.mipmaps,
            hash: value.hash,
            pixels: value.pixels.clone()
        }
    }
}

impl From<&Rgb565Image> for Rgba8Image {
    fn from(value: &Rgb565Image) -> Self {
        Self {
            dimensions: value.dimensions,
            mipmaps: value.mipmaps,
            hash: value.hash,
            pixels: value.pixels.clone()
        }
    }
}
