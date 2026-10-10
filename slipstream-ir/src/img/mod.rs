mod cmpr;

use byteorder::{BigEndian, ReadBytesExt};
pub use cmpr::*;
use slipstream_shared::{RefCursor, SlipstreamResult, error::InvalidInputError};

use crate::{encoding::ReadArrayExt, mdl0::ColorFormat};

/// Converts a 16-byte integer to an opaque RGB565 colour.
///
/// This is a separate function, as the CMPR format requires both the short and colour value.
pub fn to_rgb565(short: u16) -> glam::U8Vec4 {
    let r = ((short & 0xf8_00) >> 11) as u8; // Take 5 bits
    let g = ((short & 0x07_e0) >> 5) as u8; // Then another 6 bits
    let b = (short & 0x00_1f) as u8; // and lastly another 5 bits

    // rescale the components to the full 0-255 range.
    let r = (r << 3) | (r >> 2);
    let g = (g << 2) | (g >> 4);
    let b = (b << 3) | (b >> 2);

    glam::u8vec4(r, g, b, 255)
}

pub fn deserialize_color(
    reader: &mut RefCursor<[u8]>,
    format: ColorFormat,
) -> SlipstreamResult<glam::U8Vec4> {
    Ok(match format {
        ColorFormat::Rgb565 => {
            let short = reader.read_u16::<BigEndian>()?;
            to_rgb565(short)
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
                location: Some(reader.position()),
            }
            .into());
        }
    })
}
