use std::io::{BufWriter, Seek, Write};
use std::ops::ControlFlow;
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt};
#[cfg(feature = "egui-impls")]
use eframe::egui_wgpu;
use image::{ImageBuffer, Rgba};
use slipstream_derive::{Inspect, try_from};
use slipstream_shared::error::{InvalidInputError, UnsupportedError};
use slipstream_shared::inspect::Changes;
#[cfg(feature = "egui-impls")]
use slipstream_shared::inspect::FieldConfig;
use slipstream_shared::{RefCursor, SlipstreamError, SlipstreamResult, try_unwrap};

use crate::brres::{BFileHeader, BFileType};
use crate::encoding::Deserialize;
use crate::img::{
    CmprTexture, EncodedTexture, ExportableTexture, L4Texture, L8Texture, La4Texture, La8Texture, MAX_SAFE_PIXEL_HEIGHT, MAX_SAFE_PIXEL_WIDTH, Rgb5a3Texture, Rgb565Texture, Rgba8Texture
};
use crate::node::arena::{IrArena, IrNodeDescriptor, IrNodeKey};
use crate::node::node::{ContentSlot, IrNodeType};
use crate::visitor::{
    Visitable, Visitor, VisitorContext, VisitorContextMut, VisitorContextNode,
    VisitorContextNodeMut
};

pub const TEX0_MAGIC: [u8; 4] = [0x54, 0x45, 0x58, 0x30]; // "TEX0"

#[try_from(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[repr(u32)]
pub enum TextureFormat {
    #[inspect(rename = "4-Bit Luminance")]
    Luminance4 = 0x00,
    #[inspect(rename = "8-Bit Luminance")]
    Luminance8 = 0x01,
    #[inspect(rename = "4-Bit Luminance + Alpha")]
    LuminanceAlpha4 = 0x02,
    #[inspect(rename = "8-Bit Luminance + Alpha")]
    LuminanceAlpha8 = 0x03,
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
    Cmpr = 0x0e
}

impl Deserialize for TextureFormat {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextureData {
    Luminance4(L4Texture),
    Luminance8(L8Texture),
    LuminanceAlpha4(La4Texture),
    LuminanceAlpha8(La8Texture),
    Rgb5a3(Rgb5a3Texture),
    Rgb565(Rgb565Texture),
    Rgba8(Rgba8Texture),
    Cmpr(CmprTexture)
}

impl TextureData {
    /// The file format is automatically determined from the file extension.
    ///
    /// If you want to explicitly set the format, use [`export_to_file_with_format`] instead.
    pub fn export_to_file<P: AsRef<Path>>(&self, filepath: P) -> SlipstreamResult<()> {
        let path = filepath.as_ref();
        let format = image::ImageFormat::from_path(path)?;

        let mut file = std::fs::File::create(path)?;
        let mut buf_writer = BufWriter::new(&mut file);

        self.export(&mut buf_writer, format)
    }

    pub fn export_to_file_with_format<P: AsRef<Path>>(
        &self,
        filepath: P,
        format: image::ImageFormat
    ) -> SlipstreamResult<()> {
        let path = filepath.as_ref();
        let mut file = std::fs::File::create(path)?;
        let mut buf_writer = BufWriter::new(&mut file);

        self.export(&mut buf_writer, format)
    }

    #[cfg(feature = "egui-impls")]
    pub(super) fn draw_ui(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        use slipstream_shared::inspect::Inspect;

        ui.horizontal(|ui| {
            ui.label("Format: ");
            self.format().draw_inner(ui, cfg);
        });

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
                    .to_rgba8()
                    .expect("failed to convert texture to RGBA8 preview");

                let color_img = Arc::new(egui::ColorImage::from_rgba_unmultiplied(
                    [dims.x as usize, dims.y as usize],
                    rgba8
                        .bytes()
                        .expect("Rgba8Texture::bytes should be infallible")
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

        if ui.button("Export").clicked() {
            use crate::img;

            tracing::trace!("Opening texture export file dialog");

            if let Some(export_file) = rfd::FileDialog::new().save_file() {
                if export_file.extension().map(|ext| {
                    ext.to_string_lossy().ends_with("tex0")
                }).unwrap_or(false) {
                    // Export as TEX0 file.
                    todo!("export as tex0");
                } else {
                    // Export as regular image
                    self.export_to_file(export_file).expect("failed to export image");
                }
            }
        }

        inner.into()
    }
}

macro_rules! impl_tex_data {
    ($($ident:ident),*) => {
        impl TextureData {
            /// Exports the texture to the given image format, writing it into the given writer.
            ///
            /// *Assumes the writer is bufferred*.
            pub fn export<W: Write + Seek>(
                &self,
                buf_writer: &mut W,
                format: image::ImageFormat
            ) -> SlipstreamResult<()> {
                match &self {
                    $(
                        TextureData::$ident(x) => x.export(buf_writer, format),
                    )*
                }
            }

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

            pub fn to_rgba8(&self) -> SlipstreamResult<Rgba8Texture> {
                match &self {
                    $(
                        TextureData::$ident(x) => x.to_rgba8(),
                    )*
                }
            }

            pub fn format(&self) -> TextureFormat {
                match &self {
                    $(
                        TextureData::$ident(x) => TextureFormat::$ident,
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

impl_tex_data!(
    Luminance4,
    Luminance8,
    LuminanceAlpha4,
    LuminanceAlpha8,
    Rgb5a3,
    Rgb565,
    Rgba8,
    Cmpr
);

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Texture {
    #[inspect(tooltip = "Width and height of the texture in pixels")]
    pub size: glam::U16Vec2,
    #[inspect(rename = "Image")]
    #[inspect(with = TextureData::draw_ui, read_only)]
    pub data: TextureData
}

impl Visitable for Texture {
    fn accept(&self, node: VisitorContextNode, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor.visit_tex0(VisitorContext::new(node, self))
    }

    fn accept_mut(
        &mut self,
        node: VisitorContextNodeMut,
        visitor: &mut dyn Visitor
    ) -> std::ops::ControlFlow<()> {
        visitor.visit_tex0_mut(VisitorContextMut::new(node, self))
    }
}

#[tracing::instrument(skip_all, fields(name, parent_id))]
pub fn deserialize(
    reader: &mut RefCursor<[u8]>,
    parent_id: Option<IrNodeKey>,
    arena: &IrArena,
    name: String
) -> SlipstreamResult<IrNodeKey> {
    let subfile_header = BFileHeader::deserialize(reader, BFileType::Tex0)?;

    let flag = reader.read_u32::<BigEndian>()?;
    let pixel_width = reader.read_u16::<BigEndian>()?;
    let pixel_height = reader.read_u16::<BigEndian>()?;
    let format = TextureFormat::deserialize(reader)?;
    let mipmaps = reader.read_u32::<BigEndian>()?;
    let min_mipmap_used = reader.read_f32::<BigEndian>()?;
    let max_mipmap_used = reader.read_f32::<BigEndian>()?;
    let _unused = reader.read_u32::<BigEndian>()?;

    if pixel_width > MAX_SAFE_PIXEL_WIDTH as u16 {
        return Err(InvalidInputError {
            reason: format!("image exceeds maximum width of {MAX_SAFE_PIXEL_WIDTH} ({pixel_width})"),
            location: Some(reader.position())
        }.into())
    }

    if pixel_height > MAX_SAFE_PIXEL_HEIGHT as u16 {
        return Err(InvalidInputError {
            reason: format!("image exceeds maximum height of {MAX_SAFE_PIXEL_HEIGHT} ({pixel_height})"),
            location: Some(reader.position())
        }.into())
    }

    reader.set_position(subfile_header.get_section_start(0)?);

    let dims = glam::u16vec2(pixel_width, pixel_height);
    let data = match format {
        TextureFormat::Luminance4 => {
            TextureData::Luminance4(L4Texture::new(reader, dims, mipmaps)?)
        }
        TextureFormat::Luminance8 => {
            TextureData::Luminance8(L8Texture::new(reader, dims, mipmaps)?)
        }
        TextureFormat::LuminanceAlpha4 => {
            TextureData::LuminanceAlpha4(La4Texture::new(reader, dims, mipmaps)?)
        }
        TextureFormat::LuminanceAlpha8 => {
            TextureData::LuminanceAlpha8(La8Texture::new(reader, dims, mipmaps)?)
        }
        TextureFormat::Rgb565 => TextureData::Rgb565(Rgb565Texture::new(reader, dims, mipmaps)?),
        TextureFormat::Rgb5a3 => TextureData::Rgb5a3(Rgb5a3Texture::new(reader, dims, mipmaps)?),
        TextureFormat::Rgba8 => TextureData::Rgba8(Rgba8Texture::new(reader, dims, mipmaps)?),
        TextureFormat::Cmpr => TextureData::Cmpr(CmprTexture::new(reader, dims, mipmaps)?),
        _ => {
            tracing::error!("Image format {format:?} unimplemented, using an empty image");
            TextureData::Luminance4(L4Texture {
                dimensions: glam::u16vec2(1, 1),
                mipmaps: 1,
                pixels: vec![255; 8 * 8],
                hash: 0
            })
        }
    };

    let key = arena.insert(IrNodeDescriptor {
        label: name,
        ty: IrNodeType::Texture,
        parent: parent_id,
        children: Vec::new(),
        contents: ContentSlot::eager(Box::new(Texture {
            size: glam::u16vec2(pixel_width, pixel_height),
            data
        }))
    });

    Ok(key)
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use slipstream_shared::RefCursor;

    use crate::{node::arena::IrArena, tex0};

    // Found via fuzzing. This corrupted file seemed to cause an impossibly large allocation.
    const OOM_CORPUS: &[u8] = &[255, 0, 3, 255, 255, 255, 0, 0, 0, 0, 0, 3, 255, 255, 255, 3, 0, 0, 0, 49, 0, 0, 0, 0, 0, 0, 84, 250, 255, 251, 255, 1, 0, 0, 0, 3, 255, 255, 255, 255, 255, 5, 3, 87, 138, 255, 255, 255, 255, 255, 255, 254, 3, 0, 0, 138, 255, 255, 5, 3, 255, 87];

    #[test]
    fn oom_corpus() {
        let mut reader = RefCursor::new(Arc::from(OOM_CORPUS));
        let arena = IrArena::new();
        let _ = tex0::deserialize(&mut reader, None, &arena, String::from("oom_corpus"));
    }
}