use bitfield_struct::bitfield;
use byteorder::{BigEndian, ReadBytesExt};
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::img::{EncodedTexture, Rgba8Texture, block_count, detile, hash_pixels};
use crate::tex0::TextureFormat;

/// Converts a RGB5A3 integer format to RGBA8.
pub const fn from_rgb5a3(short: u16) -> glam::U8Vec4 {
    let has_alpha = (short >> 15) & 0x01 == 0;
    if has_alpha {
        #[bitfield(u16)]
        struct ColorBits {
            #[bits(3)]
            pub alpha: u8,
            #[bits(4)]
            pub red: u8,
            #[bits(4)]
            pub green: u8,
            #[bits(4)]
            pub blue: u8,
            _has_alpha: bool
        }

        let x = ColorBits::from_bits(short);
        glam::u8vec4(
            x.red() * 0x11,
            x.green() * 0x11,
            x.blue() * 0x11,
            x.alpha() * 0x20
        )
    } else {
        #[bitfield(u16)]
        struct ColorBits {
            #[bits(5)]
            pub red: u8,
            #[bits(5)]
            pub green: u8,
            #[bits(5)]
            pub blue: u8,
            #[bits(1)]
            _has_alpha: bool
        }

        let x = ColorBits::from_bits(short);
        glam::u8vec4(x.red() * 0x8, x.green() * 0x8, x.blue() * 0x8, 0xff)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rgb5a3Texture {
    pub(super) dimensions: glam::U16Vec2,
    pub(super) mipmaps: u32,
    pub(super) hash: u64,
    pub(super) pixels: Vec<glam::U8Vec4>
}

impl Rgb5a3Texture {
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

impl EncodedTexture for Rgb5a3Texture {
    fn format(&self) -> TextureFormat {
        TextureFormat::Rgb565
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
