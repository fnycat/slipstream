mod c;
mod c14x2;
mod cmpr;
mod luminance;
mod luminance_alpha;
mod rgb565;
mod rgb5a3;
mod rgba8;
mod tpl;

use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{Seek, Write};

pub use c::*;
pub use c14x2::*;
pub use cmpr::*;
pub use luminance::*;
pub use luminance_alpha::*;
pub use rgb5a3::*;
pub use rgb565::*;
pub use rgba8::*;
pub use tpl::*;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_shared::error::InvalidInputError;
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::encoding::ReadArrayExt;
use crate::mdl0::ColorFormat;
use crate::tex0::TextureFormat;

/// Cap maximum at the size of a 8192x8192 image. That should be more than enough.
pub const MAX_SAFE_PIXEL_WIDTH: usize = 8192;
pub const MAX_SAFE_PIXEL_HEIGHT: usize = 8192;

/// This is a separate trait from [`EncodedTexture`] to keep that trait dyn-compatible.
pub trait ExportableTexture {
    fn export<W: Write + Seek>(
        &self,
        buffered_writer: &mut W,
        format: image::ImageFormat
    ) -> SlipstreamResult<()>;
}

// We use methods instead of associated constants to ensure the trait stays dyn-compatible.
pub trait EncodedTexture {
    /// The format of this texture.
    fn format(&self) -> TextureFormat;

    /// The dimensions of the texture.
    fn dimensions(&self) -> glam::U16Vec2;

    /// Returns the inner pixel data as a raw binary buffer.
    ///
    /// This function returns `None` when the pixel content is not known yet.
    /// (Currently this is only the case for `CMPR` images as they need to be decompressed)
    fn bytes(&self) -> Option<&[u8]>;

    /// Converts this image to an `RGBA8Unorm` texture.
    fn to_rgba8(&self) -> SlipstreamResult<Rgba8Texture>;

    /// The amount of mipmaps in this texture.
    fn mipmaps(&self) -> u32;

    /// Returns the hash of the image's contents.
    fn content_hash(&self) -> u64;
}

/// Creates a hash of the image's contents.
pub fn hash_pixels(pixels: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    pixels.hash(&mut hasher);
    hasher.finish()
}

/// Computes the amount of tiles that are used for the given image.
pub const fn block_count(dimensions: glam::U16Vec2, block_size: glam::USizeVec2) -> usize {
    let blocks_x = (dimensions.x as usize).div_ceil(block_size.x);
    let blocks_y = (dimensions.y as usize).div_ceil(block_size.y);

    blocks_x as usize * blocks_y as usize
}

/// Converts a tiled image into a regular displayable image.
///
/// 'This should be ran on any images that use tiles.
pub fn detile<
    const TILE_W: usize,
    const TILE_H: usize,
    const TILE_SIZE: usize,
    P: Default + Copy
>(
    tiled: &[P],
    dims: glam::U16Vec2
) -> Vec<P> {
    let width = dims.x as usize;
    let height = dims.y as usize;

    let tiles_x = width.div_ceil(TILE_W);
    let mut out = vec![P::default(); width * height];

    let (blocks, rem) = tiled.as_chunks::<TILE_SIZE>();
    if !rem.is_empty() {
        tracing::error!("Image did not split cleanly into proper tiles. The last incomplete tile has been ignored.");
    }

    for (tile_idx, tile) in blocks.iter().enumerate() {
        let (tx, ty) = (tile_idx % tiles_x, tile_idx / tiles_x);
        for (i, &p) in tile.iter().enumerate() {
            let x = tx * TILE_W + i % TILE_W;
            let y = ty * TILE_H + i / TILE_W;

            if x < width && y < height {
                out[y * width + x] = p;
            }
        }
    }

    out
}

pub fn deserialize_color(
    reader: &mut RefCursor<[u8]>,
    format: ColorFormat
) -> SlipstreamResult<glam::U8Vec4> {
    Ok(match format {
        ColorFormat::Rgb565 => {
            let short = reader.read_u16::<BigEndian>()?;
            from_rgb565(short)
        }
        ColorFormat::Rgb8 => {
            let triad = reader.read_u24::<BigEndian>()?;
            let r = ((triad & 0xff_00_00) >> 16) as u8;
            let g = ((triad & 0x00_ff_00) >> 8) as u8;
            let b = (triad & 0x00_00_ff) as u8;

            // does not need rescaling since components are 8 bits.

            glam::u8vec4(r, g, b, 255)
        }
        ColorFormat::Rgbx32 => {
            let word = reader.read_u32::<BigEndian>()?;
            let r = ((word & 0xff_00_00_00) >> 24) as u8;
            let g = ((word & 0x00_ff_00_00) >> 16) as u8;
            let b = ((word & 0x00_00_ff_00) >> 8) as u8;
            // and we ignore the alpha??

            glam::u8vec4(r, g, b, 255)
        }
        ColorFormat::Rgba4 => {
            let short = reader.read_u16::<BigEndian>()?;
            let r = ((short & 0xf0_00) >> 12) as u8;
            let g = ((short & 0x0f_00) >> 8) as u8;
            let b = ((short & 0x00_f0) >> 4) as u8;
            let a = (short & 0x00_0f) as u8;

            // map from 4 bits to full 8-bit 0-255 range
            let r = (r << 4) | r;
            let g = (g << 4) | g;
            let b = (b << 4) | b;
            let a = (a << 4) | a;

            glam::u8vec4(r, g, b, a)
        }
        ColorFormat::Rgba6 => {
            let triad = reader.read_u24::<BigEndian>()?;
            let r = ((triad & 0xfc_00_00) >> 18) as u8;
            let g = ((triad & 0x03_f0_00) >> 12) as u8;
            let b = ((triad & 0x00_0f_c0) >> 6) as u8;
            let a = (triad & 0x00_00_3f) as u8;

            let r = (r << 2) | (r >> 4);
            let g = (g << 2) | (g >> 4);
            let b = (b << 2) | (b >> 4);
            let a = (a << 2) | (a >> 4);

            glam::u8vec4(r, g, b, a)
        }
        ColorFormat::Rgba8 => {
            let comps = reader.read_u8_array::<4>()?;
            glam::U8Vec4::from_array(comps)
        }
        ColorFormat::Invalid => {
            return Err(InvalidInputError {
                reason: String::from("cannot read color of invalid format"),
                location: Some(reader.position())
            }
            .into());
        }
    })
}
