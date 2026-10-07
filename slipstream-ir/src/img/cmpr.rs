use byteorder::{BigEndian, ReadBytesExt};
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::{
    img::{deserialize_color, to_rgb565},
    mdl0::ColorFormat,
};

pub struct CmprDescriptor {
    pub width: u16,
    pub height: u16,
    pub mipmap_count: u32,
}

pub struct CmprImage {
    pub size: glam::U16Vec2,
    pub pixels: Vec<glam::U8Vec4>,
}

impl CmprImage {
    /// When CMPR images are used as textures on the GPU, you don't need to use this function.
    /// `wgpu` natively supports DXT1 compression through the `Bc1RgbaUnorm` texture format.
    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        desc: CmprDescriptor,
    ) -> SlipstreamResult<Self> {
        const CMPR_SUBBLOCKS: usize = 4;
        const CMPR_SUBBLOCK_SIZE: usize = 4 * 4;

        let blocks = {
            let blocks_x = desc.width.div_ceil(16);
            let blocks_y = desc.height.div_ceil(16);

            blocks_x * blocks_y
        };

        let mut pixels = Vec::with_capacity(desc.width as usize * desc.height as usize);
        for _ in 0..blocks {
            // Every 16x16 image block is subdivided into 4 4x4 sub-blocks.
            for _ in 0..CMPR_SUBBLOCKS {
                // The subblock is prefixed by two shorts for the RGB565 coloor palette.
                let short0 = reader.read_u16::<BigEndian>()?;
                let short1 = reader.read_u16::<BigEndian>()?;

                let color0 = to_rgb565(short0).as_vec4();
                let color1 = to_rgb565(short1).as_vec4();

                let (color2, color3) = if short0 > short1 {
                    // Remaining palette colours are formed by interpolating a third and then two thirds of the way
                    // between the two palette entries.

                    let color2 = color0.lerp(color1, 1.0 / 3.0);
                    let color3 = color0.lerp(color1, 2.0 / 3.0);

                    (color2, color3)
                } else {
                    // The third palette entry is formed by interpolating halfway between the first two
                    // entries. The fourth entry is transparent.

                    let color2 = color0.midpoint(color1);
                    let color3 = glam::Vec4::ZERO;

                    (color2, color3)
                };

                let palette = [
                    color0.as_u8vec4(),
                    color1.as_u8vec4(),
                    color2.as_u8vec4(),
                    color3.as_u8vec4(),
                ];

                // The pixels are then stored as 2 bit indices into these blocks.

                let mut indices = reader.read_u32::<BigEndian>()?;
                for _ in 0..CMPR_SUBBLOCK_SIZE {
                    let index = indices & 0x02;
                    pixels.push(palette[index as usize]);

                    indices >>= 2;
                }
            }
        }

        Ok(Self {
            size: glam::u16vec2(desc.width, desc.height),
            pixels,
        })
    }
}
