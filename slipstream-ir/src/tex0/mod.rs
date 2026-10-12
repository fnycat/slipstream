use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
#[cfg(feature = "egui-impls")]
use eframe::egui_wgpu;
use slipstream_derive::{Inspect, try_from};
use slipstream_shared::error::UnsupportedError;
use slipstream_shared::inspect::Changes;
#[cfg(feature = "egui-impls")]
use slipstream_shared::inspect::FieldConfig;
use slipstream_shared::{RefCursor, SlipstreamResult, try_unwrap};

use crate::brres::{BFileHeader, BFileType};
use crate::encoding::Deserialize;
use crate::img::{
    CmprTexture, EncodedTexture, I4Texture, I8Texture, Ia4Texture, Ia8Texture, Rgba8UnormTexture,
};
use crate::node::arena::{IrArena, IrNodeDescriptor, IrNodeKey};
use crate::node::node::{ContentSlot, IrNodeType};
use crate::visitor::{
    Visitable, Visitor, VisitorContext, VisitorContextMut, VisitorContextNode,
    VisitorContextNodeMut,
};

pub const TEX0_MAGIC: [u8; 4] = [0x54, 0x45, 0x58, 0x30]; // "TEX0"

#[try_from(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[repr(u32)]
pub enum TextureFormat {
    I4 = 0x00,
    I8 = 0x01,
    #[inspect(rename = "IA4")]
    Ia4 = 0x02,
    #[inspect(rename = "IA8")]
    Ia8 = 0x03,
    #[inspect(rename = "RGB565")]
    Rgb565 = 0x04,
    #[inspect(rename = "RGB5A3")]
    Rgb5a3 = 0x05,
    #[inspect(rename = "RGBA8")]
    Rgba8 = 0x06,
    // there is no 0x07 apparently
    C4 = 0x08,
    C8 = 0x09,
    #[inspect(rename = "C14X2")]
    C14x2 = 0x0a,
    // also no 0x0b, 0x0c and 0x0d
    #[inspect(rename = "CMPR")]
    #[inspect(tooltip = "A compressed texture using DXT1 compression.")]
    Cmpr = 0x0e,
}

impl Deserialize for TextureFormat {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextureData {
    I4(I4Texture),
    I8(I8Texture),
    Ia4(Ia4Texture),
    Ia8(Ia8Texture),
    Cmpr(CmprTexture),
}

impl TextureData {
    #[cfg(feature = "egui-impls")]
    pub(super) fn draw_ui(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        // Check if the texture has been registered.
        let dims = self.dimensions();

        let key = ui.id().with("TexturePreview").with(self.content_hash());

        let cached = ui.data(|d| d.get_temp::<egui::TextureHandle>(key));
        let handle = match cached {
            Some(tex) => tex,
            None => {
                tracing::debug!(
                    "Generating preview for {}x{} texture (hash {})",
                    dims.x,
                    dims.y,
                    self.content_hash()
                );

                use std::sync::Arc;

                let key_str = format!("TexturePreview_{}", key.value());
                let rgba8 = self
                    .to_rgba8_unorm()
                    .expect("failed to convert texture to RGBA8 preview");

                let color_img = Arc::new(egui::ColorImage::from_rgba_unmultiplied(
                    [dims.x as usize, dims.y as usize],
                    rgba8.bytes(),
                ));

                let data = egui::ImageData::Color(color_img);

                let handle = ui.load_texture(key_str, data, egui::TextureOptions::NEAREST);
                ui.data_mut(|d| d.insert_temp(key, handle.clone()));
                handle
            }
        };

        let image = egui::Image::new(&handle)
            .tint(egui::Color32::WHITE)
            .maintain_aspect_ratio(true)
            .fit_to_exact_size(egui::vec2(256.0, 256.0));

        let egui::InnerResponse { inner, .. } =
            ui.centered_and_justified(|ui| ui.add(image.clone().sense(egui::Sense::click())));

        let modal_id = ui.id().with("PreviewModal");

        // Draw the modal before checking for clicks in the inspector.
        //
        // Otherwise the modal would register this as an outside click and immediately close again.
        if ui.data(|d| d.get_temp::<bool>(modal_id).unwrap_or(false)) {
            egui::Modal::new(modal_id)
                .frame(egui::Frame::new())
                .show(ui.ctx(), |ui| {
                    let modal_size = ui.viewport_rect().size() * 0.75;

                    let response = ui.add(image.fit_to_exact_size(modal_size));
                    if response.clicked_elsewhere() {
                        ui.data_mut(|d| d.remove_temp::<bool>(modal_id));
                    }
                });
        }

        if inner.clicked() {
            ui.data_mut(|d| d.insert_temp::<bool>(modal_id, true));
        }

        inner.into()
    }
}

macro_rules! impl_tex_data {
    ($($ident:ident),*) => {
        impl TextureData {
            pub fn mipmaps(&self) -> u32 {
                match &self {
                    $(
                        TextureData::$ident(x) => x.mipmaps(),
                    )*
                }
            }

            pub fn dimensions(&self) -> glam::U16Vec2 {
                match &self {
                    $(
                        TextureData::$ident(x) => x.dimensions(),
                    )*
                }
            }

            pub fn to_rgba8_unorm(&self) -> SlipstreamResult<Rgba8UnormTexture> {
                match &self {
                    $(
                        TextureData::$ident(x) => x.to_rgba8_unorm(),
                    )*
                }
            }

            pub fn content_hash(&self) -> u64 {
                match &self {
                    $(
                        TextureData::$ident(x) => x.content_hash(),
                    )*
                }
            }
        }
    }
}

impl_tex_data!(I4, I8, Ia4, Ia8, Cmpr);

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Texture {
    #[inspect(tooltip = "The format used for the pixels in the texture")]
    pub format: TextureFormat,
    #[inspect(tooltip = "Width and height of the texture in pixels")]
    pub size: glam::U16Vec2,
    #[inspect(rename = "Image")]
    #[inspect(with = "TextureData::draw_ui")]
    pub data: TextureData,
    #[inspect(ignore)]
    #[cfg(feature = "egui-impls")]
    pub egui_texture_id: Option<egui::TextureId>,
}

impl Texture {
    #[cfg(feature = "egui-impls")]
    pub fn get_egui_texture_id(&self) -> Option<egui::TextureId> {
        self.egui_texture_id
    }

    /// Generates a texture to be used in the UI.
    ///
    /// egui only supports `Rgba8Unorm` textures, so all textures are converted to this format first.
    #[cfg(feature = "egui-impls")]
    pub fn create_egui_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut egui_wgpu::Renderer,
    ) -> SlipstreamResult<egui::TextureId> {
        use wgpu::util::DeviceExt;

        let rgba8_unorm = self.data.to_rgba8_unorm()?;
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("TEX0 texture"),
                size: wgpu::Extent3d {
                    width: self.size.x as u32,
                    height: self.size.y as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: self.data.mipmaps(),
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            bytemuck::cast_slice(rgba8_unorm.pixels()),
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("TEX0 texture view"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: Some(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(self.data.mipmaps()),
            base_array_layer: 0,
            array_layer_count: None,
        });

        // Use `Nearest` filtering so small textures won't look blurry.
        let id = renderer.register_native_texture(device, &texture_view, wgpu::FilterMode::Nearest);
        self.egui_texture_id = Some(id);
        Ok(id)
    }
}

impl Visitable for Texture {
    fn accept(&self, node: VisitorContextNode, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor.visit_tex0(VisitorContext::new(node, self))
    }

    fn accept_mut(
        &mut self,
        node: VisitorContextNodeMut,
        visitor: &mut dyn Visitor,
    ) -> std::ops::ControlFlow<()> {
        visitor.visit_tex0_mut(VisitorContextMut::new(node, self))
    }
}

#[tracing::instrument(skip_all, fields(name, parent_id))]
pub fn deserialize(
    reader: &mut RefCursor<[u8]>,
    parent_id: IrNodeKey,
    arena: &IrArena,
    name: String,
) -> SlipstreamResult<IrNodeKey> {
    let subfile_header = BFileHeader::deserialize(reader, BFileType::Tex0)?;

    let flag = reader.read_u32::<BigEndian>()?;
    let pixel_width = reader.read_u16::<BigEndian>()?;
    let pixel_height = reader.read_u16::<BigEndian>()?;
    let format = TextureFormat::deserialize(reader)?;
    let mipmap_count = reader.read_u32::<BigEndian>()?;
    let min_mipmap_used = reader.read_f32::<BigEndian>()?;
    let max_mipmap_used = reader.read_f32::<BigEndian>()?;
    let _unused = reader.read_u32::<BigEndian>()?;

    reader.set_position(subfile_header.get_section_start(0)?);

    let dims = glam::u16vec2(pixel_width, pixel_height);
    let data = match format {
        TextureFormat::I4 => TextureData::I4(I4Texture::new(reader, dims, mipmap_count)?),
        TextureFormat::I8 => TextureData::I8(I8Texture::new(reader, dims, mipmap_count)?),
        TextureFormat::Ia4 => TextureData::Ia4(Ia4Texture::new(reader, dims, mipmap_count)?),
        TextureFormat::Ia8 => TextureData::Ia8(Ia8Texture::new(reader, dims, mipmap_count)?),
        TextureFormat::Cmpr => TextureData::Cmpr(CmprTexture::new(reader, dims, mipmap_count)?),
        _ => {
            tracing::error!("Image format {format:?} unimplemented, using an empty image");
            TextureData::I4(I4Texture {
                dimensions: glam::u16vec2(1, 1),
                mipmaps: 1,
                pixels: vec![255; 8 * 8],
                hash: 0,
            })
        }
    };

    let key = arena.insert(IrNodeDescriptor {
        label: name,
        ty: IrNodeType::Texture,
        parent: Some(parent_id),
        children: Vec::new(),
        contents: ContentSlot::eager(Box::new(Texture {
            format,
            size: glam::u16vec2(pixel_width, pixel_height),
            data,

            #[cfg(feature = "egui-impls")]
            egui_texture_id: None,
        })),
    });

    Ok(key)
}
