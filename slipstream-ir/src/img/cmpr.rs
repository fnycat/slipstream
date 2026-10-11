use byteorder::{BigEndian, ReadBytesExt};
use slipstream_shared::{RefCursor, SlipstreamResult, verify};

use crate::img::{EncodedTexture, Rgba8UnormTexture, deserialize_color, to_rgb565};
use crate::tex0::TextureFormat;

#[derive(Debug, Clone, PartialEq)]
pub struct CmprTexture {
    pub dimensions: glam::U16Vec2,
    pub mipmaps: u32,
    // As `wgpu` natively supports this format, we will simply store a reference to the reader
    // instead of deserializing the texture data.
    pub raw: RefCursor<[u8]>,
}

impl CmprTexture {
    /// How many CMPR subblocks are there in one general image block?
    pub const SUBBLOCKS: usize = 4;
    /// How many pixels are in a single CMPR subblock?
    pub const SUBBLOCK_SIZE: usize = 4 * 4;
    /// The format of this texture when uploaded to the GPU.
    #[cfg(feature = "egui-impls")]
    pub const SHADER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bc1RgbaUnorm;

    pub const fn subblock_count(&self) -> usize {
        let dims = self.dimensions;
        let blocks_x = dims.x.div_ceil(8);
        let blocks_y = dims.y.div_ceil(8);

        blocks_x as usize * blocks_y as usize
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

        let data_len = self.data_len();
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("CMPR DXT1 texture"),
                format: Self::SHADER_FORMAT,
                dimension: wgpu::TextureDimension::D2,
                size: wgpu::Extent3d {
                    width: self.dimensions().x as u32,
                    height: self.dimensions().y as u32,
                    depth_or_array_layers: 1,
                },
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[Self::SHADER_FORMAT],
                mip_level_count: self.mipmaps(),
                sample_count: 1,
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            bytemuck::cast_slice(self.raw.slice(..data_len)),
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("CMPR DXT1 texture view"),
            format: Some(Self::SHADER_FORMAT),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
            base_mip_level: 0,
            mip_level_count: Some(self.mipmaps()),
            base_array_layer: 0,
            array_layer_count: None,
        });

        (texture, view)
    }

    /// When CMPR images are used as textures on the GPU, you don't need to use this function.
    /// `wgpu` natively supports DXT1 compression through the `Bc1RgbaUnorm` texture format.
    pub fn new(
        reader: &mut RefCursor<[u8]>,
        dimensions: glam::U16Vec2,
        mipmap_count: u32,
    ) -> SlipstreamResult<Self> {
        Ok(Self {
            dimensions,
            mipmaps: mipmap_count,
            raw: reader.clone(),
        })
    }
}

impl EncodedTexture for CmprTexture {
    fn format(&self) -> TextureFormat {
        TextureFormat::Cmpr
    }

    fn dimensions(&self) -> glam::U16Vec2 {
        self.dimensions
    }

    fn data_len(&self) -> usize {
        let blocks = self.subblock_count();
        blocks * 32
    }

    fn to_rgba8_unorm(&self) -> SlipstreamResult<Rgba8UnormTexture> {
        let mut reader = self.raw.clone();
        let reader_start = reader.position();

        let dims = self.dimensions();
        let blocks = self.subblock_count();

        let mut pixels = Vec::with_capacity(dims.x as usize * dims.y as usize);
        for _ in 0..blocks {
            // Every 8x8 image block is subdivided into 4 4x4 sub-blocks.
            for _ in 0..Self::SUBBLOCKS {
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
                for _ in 0..Self::SUBBLOCK_SIZE {
                    let index = indices & 0x02;
                    pixels.push(palette[index as usize]);

                    indices >>= 2;
                }
            }
        }

        let actual_len = reader.position() - reader_start;
        let expected_len = self.data_len() as u64;

        verify!(
            actual_len == expected_len,
            "incorrectly decompressed CMPR image, expected to read {expected_len} bytes but read {actual_len}"
        );

        Ok(Rgba8UnormTexture::new(self.dimensions(), pixels))
    }

    fn mipmaps(&self) -> u32 {
        self.mipmaps
    }
}
