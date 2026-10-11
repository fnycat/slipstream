use byteorder::{BigEndian, ReadBytesExt};
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::img::{Rgba8Image, deserialize_color, to_rgb565};
use crate::mdl0::ColorFormat;

#[derive(Debug, Clone, PartialEq)]
pub struct CmprDescriptor {
    pub dimensions: glam::U16Vec2,
    pub mipmap_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CmprImage {
    pub desc: CmprDescriptor,

    pub data_start: u64,
    pub raw: RefCursor<[u8]>,
}

impl CmprImage {
    pub fn dimensions(&self) -> glam::U16Vec2 {
        self.desc.dimensions
    }

    /// Computes the size in bytes of the image data.
    ///
    /// This includes all different mip levels.
    pub fn data_size(&self) -> usize {
        todo!()
    }

    /// Creates a `wgpu` texture from this image. The image will be uploaded using the [`Bc1RgbaUnorm`]
    /// format.
    ///
    /// [`Bc1RgbaUnorm`]: wgpu::TextureFormat::Bc1RgbaUnorm
    #[cfg(feature = "egui-impls")]
    pub fn create_wgpu_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        use wgpu::util::DeviceExt;

        self.raw.set_position(self.data_start);
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("CMPR DXT1 texture"),
                format: wgpu::TextureFormat::Bc1RgbaUnorm,
                dimension: wgpu::TextureDimension::D2,
                size: wgpu::Extent3d {
                    width: self.desc.dimensions.x as u32,
                    height: self.desc.dimensions.y as u32,
                    depth_or_array_layers: 1,
                },
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[wgpu::TextureFormat::Bc1RgbaUnorm],
                mip_level_count: 1,
                sample_count: 1,
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            bytemuck::cast_slice(&self.raw.slice()),
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("CMPR DXT1 texture view"),
            format: Some(wgpu::TextureFormat::Bc1RgbaUnorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        (texture, view)
    }

    /// Decompresses the image, converting it to RGBA8 Unorm format.
    pub fn to_rgba8_unorm(&self) -> Rgba8Image {
        const CMPR_SUBBLOCKS: usize = 4;
        const CMPR_SUBBLOCK_SIZE: usize = 4 * 4;

        let blocks = {
            let blocks_x = self.size.x.div_ceil(16);
            let blocks_y = self.size.y.div_ceil(16);

            blocks_x * blocks_y
        };

        let mut pixels = Vec::with_capacity(self.size.x as usize * self.size.y as usize);
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
    }

    /// When CMPR images are used as textures on the GPU, you don't need to use this function.
    /// `wgpu` natively supports DXT1 compression through the `Bc1RgbaUnorm` texture format.
    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        desc: CmprDescriptor,
    ) -> SlipstreamResult<Self> {
        Ok(Self {
            size: glam::u16vec2(desc.width, desc.height),
            pixels,
        })
    }
}
