use std::io::ErrorKind::InvalidInput;
use std::io::{Read, Seek, Write};

use byteorder::ReadBytesExt;
use glam::Vec2Swizzles;
use slipstream_shared::error::InvalidInputError;
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::tpl::{
    EncodedImage, ExportableImage, ImageFormat, MAX_SAFE_PIXEL_WIDTH, Rgba8Image, block_count,
    detile, hash_pixels
};

/// Single channel 4-bit intensity values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct La4Image {
    pub(super) dimensions: glam::U16Vec2,
    pub(super) mipmaps: u32,
    /// Intensity values. The values have already been multiplied by `0x11` compared
    /// to the disk format.
    pub(super) pixels: Vec<glam::U8Vec2>,
    pub(super) hash: u64
}

impl La4Image {
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
            for _ in 0..(Self::BLOCK_W * Self::BLOCK_H / 2) {
                let byte = reader.read_u8()?;
                let a = (byte & 0x0F) * 0x11;
                let rgb = ((byte >> 4) & 0x0F) * 0x11;

                tiles.push(glam::u8vec2(rgb, a))
            }
        }

        let pixels = detile::<8, 4, 32, _>(&tiles, dimensions);
        let hash = hash_pixels(bytemuck::cast_slice(&pixels));

        Ok(Self {
            dimensions,
            mipmaps,
            pixels,
            hash
        })
    }
}

impl EncodedImage for La4Image {
    // #[cfg(feature = "egui-impls")]
    // fn create_wgpu_texture(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> (wgpu::Texture, wgpu::TextureView) {
    //     use wgpu::util::DeviceExt;

    //     const SHADER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg8Unorm;

    //     let dims = self.dimensions();
    //     let rgba = self.to_rgba8().expect("La4 to Rgba8 conversion should not fail");

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
    //     }, wgpu::util::TextureDataOrder::LayerMajor, rgba.bytes().expect("Rgba8 always has available bytes"));

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
    //         // swizzle: wgpu::TextureComponentSwizzle {
    //         //     r: wgpu::ComponentSwizzle::R,
    //         //     g: wgpu::ComponentSwizzle::R,
    //         //     b: wgpu::ComponentSwizzle::R,
    //         //     a: wgpu::ComponentSwizzle::G
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
        let out = self
            .pixels
            .iter()
            .copied()
            .map(|i| i.xxxy())
            .collect::<Vec<_>>();

        Ok(Rgba8Image::from(self))
    }

    fn mipmaps(&self) -> u32 {
        self.mipmaps
    }

    fn content_hash(&self) -> u64 {
        self.hash
    }
}

impl ExportableImage for La4Image {
    // fn import<R: Read>(reader: &mut R) -> SlipstreamResult<Self> {
    //     todo!();
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
            self.bytes()
                .expect("La4Texture::bytes should be infallible"),
            dims.x as u32,
            dims.y as u32,
            image::ExtendedColorType::La4,
            format
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct La8Image {
    pub(super) dimensions: glam::U16Vec2,
    pub(super) mipmaps: u32,
    pub(super) pixels: Vec<glam::U8Vec2>,
    pub(super) hash: u64
}

impl La8Image {
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
            for _ in 0..(Self::BLOCK_W * Self::BLOCK_H) {
                let a = reader.read_u8()?;
                let rgb = reader.read_u8()?;

                tiled.push(glam::u8vec2(rgb, a));
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

impl EncodedImage for La8Image {
    // #[cfg(feature = "egui-impls")]
    // fn create_wgpu_texture(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> (wgpu::Texture, wgpu::TextureView) {
    //     use wgpu::util::DeviceExt;

    //     const SHADER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg8Unorm;

    //     let dims = self.dimensions();
    //     let rgba = self.to_rgba8().expect("La8 to Rgba8 conversion should not fail");

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
    //     }, wgpu::util::TextureDataOrder::LayerMajor, rgba.bytes().expect("Rgba8 always has available bytes"));

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
    //         // TODO: Implement using swizzling when it has landed in `wgpu` and `eframe`.
    //         // swizzle: wgpu::TextureComponentSwizzle {
    //         //     r: wgpu::ComponentSwizzle::R,
    //         //     g: wgpu::ComponentSwizzle::R,
    //         //     b: wgpu::ComponentSwizzle::R,
    //         //     a: wgpu::ComponentSwizzle::G
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

impl ExportableImage for La8Image {
    // fn import<R: Read>(reader: &mut R) -> SlipstreamResult<Self> {
    //     todo!();
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
            self.bytes()
                .expect("La8Texture::bytes should be infallible"),
            dims.x as u32,
            dims.y as u32,
            image::ExtendedColorType::La8,
            format
        )?;

        Ok(())
    }
}
