use std::io::{Seek, Write};

use bitfield_struct::bitfield;
use byteorder::{BigEndian, ReadBytesExt};
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::tpl::{
    EncodedImage, ExportableImage, ImageFormat, Rgba8Image, block_count, detile, hash_pixels
};

/// Converts an RGB5A3 integer format to RGBA8.
pub const fn from_rgb5a3(short: u16) -> glam::U8Vec4 {
    let has_alpha = short & 0x8000 == 0;
    if has_alpha {
        #[bitfield(u16)]
        struct ColorChannels {
            #[bits(4)]
            pub blue: u8,
            #[bits(4)]
            pub green: u8,
            #[bits(4)]
            pub red: u8,
            #[bits(3)]
            pub alpha: u8,
            pub alpha_disabled: bool
        }

        let channels = ColorChannels::from_bits(short);
        debug_assert!(!channels.alpha_disabled());

        glam::u8vec4(
            channels.red() * 0x11,
            channels.green() * 0x11,
            channels.blue() * 0x11,
            channels.alpha() * 0x20
        )
    } else {
        #[bitfield(u16)]
        struct ColorChannels {
            #[bits(5)]
            pub blue: u8,
            #[bits(5)]
            pub green: u8,
            #[bits(5)]
            pub red: u8,
            pub alpha_disabled: bool
        }

        let channels = ColorChannels::from_bits(short);
        debug_assert!(channels.alpha_disabled());

        glam::u8vec4(
            channels.red() * 0x8,
            channels.green() * 0x8,
            channels.blue() * 0x8,
            0xff
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rgb5a3Image {
    pub(super) dimensions: glam::U16Vec2,
    pub(super) mipmaps: u32,
    pub(super) hash: u64,
    pub(super) pixels: Vec<glam::U8Vec4>
}

impl Rgb5a3Image {
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
                let color = from_rgb5a3(reader.read_u16::<BigEndian>()?);
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

impl EncodedImage for Rgb5a3Image {
    fn format(&self) -> ImageFormat {
        ImageFormat::Rgb565
    }

    fn dimensions(&self) -> glam::U16Vec2 {
        self.dimensions
    }

    fn bytes(&self) -> Option<&[u8]> {
        Some(bytemuck::cast_slice(&self.pixels))
    }

    fn to_rgba8(&self) -> SlipstreamResult<Rgba8Image> {
        Ok(Rgba8Image::from(self))
    }

    fn mipmaps(&self) -> u32 {
        self.mipmaps
    }

    fn content_hash(&self) -> u64 {
        self.hash
    }
}

impl ExportableImage for Rgb5a3Image {
    #[tracing::instrument(skip_all, fields(format))]
    fn export<W: Write + Seek>(
        &self,
        buffered_writer: &mut W,
        format: image::ImageFormat
    ) -> SlipstreamResult<()> {
        let dims = self.dimensions();

        // The image data is already converted to RGBA8 while reading, so we can simply write it as an Rgba8 image.
        image::write_buffer_with_format(
            buffered_writer,
            self.bytes()
                .expect("Rgb5a3Texture::bytes should be infallible"),
            dims.x as u32,
            dims.y as u32,
            image::ExtendedColorType::Rgba8,
            format
        )?;

        Ok(())
    }
}
