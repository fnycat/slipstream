use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{Read, Seek, Write};

use byteorder::ReadBytesExt;

use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::tpl::{
    EncodedImage, ExportableImage, ImageFormat, Rgba8Image, block_count, detile, hash_pixels
};

/// Single channel 4-bit intensity values.
#[derive(Debug, Clone, PartialEq)]
pub struct L4Image {
    pub dimensions: glam::U16Vec2,
    pub mipmaps: u32,
    /// Intensity values. The values have already been multiplied by `0x11` compared
    /// to the disk format.
    pub pixels: Vec<u8>,

    /// A hash of the pixel's contents.
    pub hash: u64
}

impl L4Image {
    pub const BLOCK_W: usize = 8;
    pub const BLOCK_H: usize = 8;
    pub const BLOCK_SIZE: glam::USizeVec2 = glam::usizevec2(Self::BLOCK_W, Self::BLOCK_H);

    pub fn new(
        reader: &mut RefCursor<[u8]>,
        dimensions: glam::U16Vec2,
        mipmaps: u32
    ) -> SlipstreamResult<Self> {
        let blocks = block_count(dimensions, Self::BLOCK_SIZE);

        // Decode into tiles.
        let mut tiles = Vec::with_capacity(dimensions.x as usize * dimensions.y as usize);
        for _ in 0..blocks {
            // Decode an 8x8 pixel block.
            for _ in 0..(Self::BLOCK_W * Self::BLOCK_H / 2) {
                let pair = reader.read_u8()?;

                // multiply by 0x11 to map from 4 bit to 8 bit range.
                tiles.push((pair & 0x0F) * 0x11);
                tiles.push(((pair >> 4) & 0x0F) * 0x11);
            }
        }

        // Then remove the tile pattern
        let pixels = detile::<8, 8, 64, _>(&tiles, dimensions);
        let hash = hash_pixels(&pixels);

        Ok(Self {
            dimensions,
            mipmaps,
            pixels,
            hash
        })
    }
}

impl EncodedImage for L4Image {
    // #[cfg(feature = "egui-impls")]
    // fn create_wgpu_texture(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> (wgpu::Texture, wgpu::TextureView) {
    //     use wgpu::util::DeviceExt;

    //     const SHADER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    //     let dims = self.dimensions();
    //     let rgba8 = self.to_rgba8().expect("Conversion from L4 to Rgba8 should not fail");

    //     let texture = device.create_texture_with_data(queue, &wgpu::TextureDescriptor {
    //         label: Some("CMPR texture"),
    //         size: wgpu::Extent3d {
    //             width: dims.x as u32,
    //             height: dims.y as u32,
    //             depth_or_array_layers: 1
    //         },
    //         mip_level_count: 1,
    //         sample_count: 1,
    //         dimension: wgpu::TextureDimension::D2,
    //         format: SHADER_FORMAT,
    //         usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    //         view_formats: &[SHADER_FORMAT]
    //     }, wgpu::util::TextureDataOrder::LayerMajor, rgba8.bytes().expect("Rgba8 always has available bytes"));

    //     let view = texture.create_view(&wgpu::TextureViewDescriptor {
    //         label: Some("CMPR texture view"),
    //         format: Some(SHADER_FORMAT),
    //         dimension: Some(wgpu::TextureViewDimension::D2),
    //         usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
    //         aspect: wgpu::TextureAspect::All,
    //         base_mip_level: 0,
    //         mip_level_count: None,
    //         base_array_layer: 0,
    //         array_layer_count: None,
    //         // TODO: Implement using swizzling instead of RGBA conversion when it is implemented in `wgpu` and `eframe` has updated to the `wgpu` version.
    //         // swizzle: wgpu::TextureComponentSwizzle {
    //         //     r: wgpu::ComponentSwizzle::R,
    //         //     g: wgpu::ComponentSwizzle::R,
    //         //     b: wgpu::ComponentSwizzle::R,
    //         //     a: wgpu::ComponentSwizzle::One
    //         // }
    //     });

    //     (texture, view)
    // }

    fn format(&self) -> ImageFormat {
        ImageFormat::Luminance4
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

impl ExportableImage for L4Image {
    // fn import<R: Read>(reader: &mut R) -> SlipstreamResult<Self> {
    //     todo!()
    // }

    #[tracing::instrument(skip_all, fields(format))]
    fn export<W: Write + Seek>(
        &self,
        buffered_writer: &mut W,
        format: image::ImageFormat
    ) -> SlipstreamResult<()> {
        let dims = self.dimensions();

        image::write_buffer_with_format(
            buffered_writer,
            self.bytes().expect("L4Texture::bytes should be infallible"),
            dims.x as u32,
            dims.y as u32,
            image::ExtendedColorType::L4,
            format
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct L8Image {
    pub(super) dimensions: glam::U16Vec2,
    pub(super) mipmaps: u32,
    pub(super) pixels: Vec<u8>,
    pub(super) hash: u64
}

impl L8Image {
    pub const BLOCK_W: usize = 8;
    pub const BLOCK_H: usize = 4;
    pub const BLOCK_SIZE: glam::USizeVec2 = glam::usizevec2(Self::BLOCK_W, Self::BLOCK_H);

    pub fn new(
        reader: &mut RefCursor<[u8]>,
        dimensions: glam::U16Vec2,
        mipmaps: u32
    ) -> SlipstreamResult<Self> {
        let blocks = block_count(dimensions, Self::BLOCK_SIZE);

        let mut tiles = Vec::with_capacity(dimensions.x as usize * dimensions.y as usize);
        for _ in 0..blocks {
            // Decode an 8x8 pixel block.
            for _ in 0..Self::BLOCK_W * Self::BLOCK_H {
                tiles.push(reader.read_u8()?);
            }
        }

        let pixels = detile::<8, 4, 32, _>(&tiles, dimensions);
        let hash = hash_pixels(&pixels);

        Ok(Self {
            dimensions,
            mipmaps,
            pixels,
            hash
        })
    }
}

impl EncodedImage for L8Image {
    // #[cfg(feature = "egui-impls")]
    // fn create_wgpu_texture(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> (wgpu::Texture, wgpu::TextureView) {
    //     use wgpu::util::DeviceExt;

    //     const SHADER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    //     let dims = self.dimensions();
    //     let rgba8 = self.to_rgba8().expect("Conversion from L8 to Rgba8 should not fail");

    //     let texture = device.create_texture_with_data(queue, &wgpu::TextureDescriptor {
    //         label: Some("CMPR texture"),
    //         size: wgpu::Extent3d {
    //             width: dims.x as u32,
    //             height: dims.y as u32,
    //             depth_or_array_layers: 1
    //         },
    //         mip_level_count: 1,
    //         sample_count: 1,
    //         dimension: wgpu::TextureDimension::D2,
    //         format: SHADER_FORMAT,
    //         usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    //         view_formats: &[SHADER_FORMAT]
    //     }, wgpu::util::TextureDataOrder::LayerMajor, rgba8.bytes().expect("Rgba8 always has available bytes"));

    //     let view = texture.create_view(&wgpu::TextureViewDescriptor {
    //         label: Some("CMPR texture view"),
    //         format: Some(SHADER_FORMAT),
    //         dimension: Some(wgpu::TextureViewDimension::D2),
    //         usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
    //         aspect: wgpu::TextureAspect::All,
    //         base_mip_level: 0,
    //         mip_level_count: None,
    //         base_array_layer: 0,
    //         array_layer_count: None,
    //         // TODO: Implement using swizzling instead of RGBA conversion when it is implemented in `wgpu` and `eframe` has updated to the `wgpu` version.
    //         // swizzle: wgpu::TextureComponentSwizzle {
    //         //     r: wgpu::ComponentSwizzle::R,
    //         //     g: wgpu::ComponentSwizzle::R,
    //         //     b: wgpu::ComponentSwizzle::R,
    //         //     a: wgpu::ComponentSwizzle::One
    //         // }
    //     });

    //     (texture, view)
    // }

    fn format(&self) -> ImageFormat {
        ImageFormat::Luminance8
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

impl ExportableImage for L8Image {
    // fn import<R: Read>(reader: &mut R) -> SlipstreamResult<Self> {
    //     todo!()
    // }

    #[tracing::instrument(skip_all, fields(format))]
    fn export<W: Write + Seek>(
        &self,
        buffered_writer: &mut W,
        format: image::ImageFormat
    ) -> SlipstreamResult<()> {
        let dims = self.dimensions();

        image::write_buffer_with_format(
            buffered_writer,
            self.bytes().expect("L8Texture::bytes should be infallible"),
            dims.x as u32,
            dims.y as u32,
            image::ExtendedColorType::L8,
            format
        )?;

        Ok(())
    }
}
