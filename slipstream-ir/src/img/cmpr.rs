use byteorder::{BigEndian, ReadBytesExt};
use slipstream_shared::{RefCursor, SlipstreamResult, verify};

use crate::img::{
    EncodedTexture, Rgba8UnormTexture, deserialize_color, detile, hash_pixels, to_rgb565,
};
use crate::tex0::TextureFormat;

#[derive(Debug, Clone, PartialEq)]
pub struct CmprTexture {
    pub dimensions: glam::U16Vec2,
    pub mipmaps: u32,
    // As `wgpu` natively supports this format, we will simply store a reference to the reader
    // instead of deserializing the texture data.
    pub raw: RefCursor<[u8]>,
    pub hash: u64,
}

impl CmprTexture {
    /// How many CMPR subblocks are there in one general image block?
    pub const SUBBLOCKS: usize = 4;
    /// How many pixels are in a single CMPR subblock?
    pub const SUBBLOCK_SIZE: usize = 4 * 4;
    /// Size in bytes of a CMPR subblock.
    pub const SUBBLOCK_BYTE_SIZE: usize = 8;
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
        let blocks_x = dimensions.x.div_ceil(8);
        let blocks_y = dimensions.y.div_ceil(8);

        let block_count = blocks_x as usize * blocks_y as usize;
        let subblock_count = Self::SUBBLOCKS * block_count;
        let data_len = subblock_count * Self::SUBBLOCK_BYTE_SIZE;

        let hash = hash_pixels(reader.slice(..data_len));

        Ok(Self {
            dimensions,
            mipmaps: mipmap_count,
            raw: reader.clone(),
            hash,
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

        let (w, h) = (dims.x as usize, dims.y as usize);
        let mut pixels = vec![glam::U8Vec4::ZERO; w * h];

        for tile_y in 0..h.div_ceil(8) {
            for tile_x in 0..w.div_ceil(8) {
                for sub in 0..4 {
                    let orig_x = tile_x * 8 + (sub % 2) * 4;
                    let orig_y = tile_y * 8 + (sub / 2) * 4;

                    // The subblock is prefixed by two shorts for the RGB565 color palette.
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

                    let mut indices = reader.read_u32::<BigEndian>()?;
                    for i in 0..16 {
                        let index = (indices >> 30) & 0b11;
                        indices <<= 2;

                        let (x, y) = (orig_x + i % 4, orig_y + i / 4);
                        if x < w && y < h {
                            pixels[y * w + x] = palette[index as usize];
                        }
                    }
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

    fn content_hash(&self) -> u64 {
        self.hash
    }
}
