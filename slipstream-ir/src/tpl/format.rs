use std::io::{BufWriter, Seek, Write};
use std::path::Path;
use std::sync::Arc;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use slipstream_derive::{Inspect, try_from};
use slipstream_shared::cursor::MutCursor;
use slipstream_shared::{RefCursor, SlipstreamResult};

#[cfg(feature = "egui-impls")]
use slipstream_shared::inspect::{Changes, FieldConfig};

use crate::encoding::Deserialize;
use crate::tpl::{
    CmprImage, EncodedImage, ExportableImage, L4Image, L8Image, La4Image, La8Image, Rgb5a3Image,
    Rgb565Image, Rgba8Image
};

pub const SUPPORTED_EXPORT_FORMATS: [&str; 4] = ["png", "jpeg", "jpg", "webp"];

#[try_from(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[repr(u32)]
pub enum ImageFormat {
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

impl ImageFormat {
    pub const fn is_paletted(&self) -> bool {
        matches!(self, Self::C4 | Self::C8 | Self::C14x2)
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(*self as u32)?;
        Ok(())
    }
}

impl Deserialize for ImageFormat {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageData {
    Luminance4(L4Image),
    Luminance8(L8Image),
    LuminanceAlpha4(La4Image),
    LuminanceAlpha8(La8Image),
    Rgb5a3(Rgb5a3Image),
    Rgb565(Rgb565Image),
    Rgba8(Rgba8Image),
    Cmpr(CmprImage)
}

impl ImageData {
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
    pub fn draw_ui(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        use slipstream_shared::inspect::Inspect;

        ui.horizontal(|ui| {
            ui.label("Format: ");
            self.format().draw_inner(ui, cfg);
        });

        // Check if the texture has been registered.
        let dims = self.dimensions();

        let key = ui
            .scope_id()
            .with("TexturePreview")
            .with(self.content_hash());

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

        let modal_id = ui.scope_id().with("PreviewModal");

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
            use slipstream_shared::file_dialog::{FileDialogKind, pick_file_dialog};

            tracing::trace!("Opening texture export file dialog");

            let rgba_image = self
                .to_rgba8()
                .expect("failed to convert image to RGBA8 for export");

            // Creates the file dialog to export a file.
            //
            // On native this is polled within a separate thread to prevent blocking the UI thread.
            // On WASM this future will return immediately and the file dialog will only open when the
            // editor attempts to write to the file.
            let fut = rfd::AsyncFileDialog::new()
                // .set_title("Export image")
                // .add_filter("", &SUPPORTED_EXPORT_FORMATS)
                .save_file();

            #[cfg(not(target_arch = "wasm32"))]
            {
                // Polls in a new thread so the UI thread isn't blocked.
                //
                // We copy the RGBA8 image data into the new thread because the thread
                // will last multiple frames and cannot hold a reference into the node.
                pick_file_dialog(FileDialogKind::Save, async move |file: rfd::FileHandle| {
                    tracing::info!("Exporting image to `{:?}`", file.path());

                    rgba_image
                        .export_to_file(file.path())
                        .expect("failed to export file");
                });
            }

            #[cfg(target_arch = "wasm32")]
            {
                use std::io::Cursor;

                pick_file_dialog(FileDialogKind::Save, async |file: rfd::FileHandle| {
                    let filename = file.file_name();
                    tracing::info!("WASM file dialog returned with file: `{filename}`");

                    let Ok(format) = image::ImageFormat::from_path(filename) else {
                        tracing::error!("Failed to guess export format from file path");
                        return;
                    };

                    let mut writer = BufWriter::new(Cursor::new(Vec::new()));
                    rgba_image.export(&mut writer, format);

                    writer
                        .flush()
                        .expect("Failed to flush image export buf writer");

                    file.write(writer.buffer()).await;
                });
            }
        }

        inner.into()
    }

    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        dims: glam::U16Vec2,
        mipmaps: u32,
        format: ImageFormat
    ) -> SlipstreamResult<Self> {
        Ok(match format {
            ImageFormat::Luminance4 => ImageData::Luminance4(L4Image::new(reader, dims, mipmaps)?),
            ImageFormat::Luminance8 => ImageData::Luminance8(L8Image::new(reader, dims, mipmaps)?),
            ImageFormat::LuminanceAlpha4 => {
                ImageData::LuminanceAlpha4(La4Image::new(reader, dims, mipmaps)?)
            }
            ImageFormat::LuminanceAlpha8 => {
                ImageData::LuminanceAlpha8(La8Image::new(reader, dims, mipmaps)?)
            }
            ImageFormat::Rgb565 => ImageData::Rgb565(Rgb565Image::new(reader, dims, mipmaps)?),
            ImageFormat::Rgb5a3 => ImageData::Rgb5a3(Rgb5a3Image::new(reader, dims, mipmaps)?),
            ImageFormat::Rgba8 => ImageData::Rgba8(Rgba8Image::new(reader, dims, mipmaps)?),
            ImageFormat::Cmpr => ImageData::Cmpr(CmprImage::new(reader, dims, mipmaps)?),
            _ => {
                tracing::error!("Image format {format:?} unimplemented, using an empty image");
                ImageData::Luminance4(L4Image {
                    dimensions: glam::u16vec2(1, 1),
                    mipmaps: 1,
                    pixels: Arc::from(vec![255; 8 * 8]),
                    hash: 0
                })
            }
        })
    }
}

macro_rules! impl_img_data {
    ($($ident:ident),*) => {
        impl ImageData {
            /// Exports the texture to the given image format, writing it into the given writer.
            ///
            /// *Assumes the writer is buffered*.
            pub fn export<W: Write + Seek>(
                &self,
                buf_writer: &mut W,
                format: image::ImageFormat
            ) -> SlipstreamResult<()> {
                match &self {
                    $(
                        ImageData::$ident(x) => x.export(buf_writer, format),
                    )*
                }
            }

            pub fn mipmaps(&self) -> u32 {
                match &self {
                    $(
                        ImageData::$ident(x) => x.mipmaps(),
                    )*
                }
            }

            pub fn dimensions(&self) -> glam::U16Vec2 {
                match &self {
                    $(
                        ImageData::$ident(x) => x.dimensions(),
                    )*
                }
            }

            pub fn to_rgba8(&self) -> SlipstreamResult<Rgba8Image> {
                match &self {
                    $(
                        ImageData::$ident(x) => x.to_rgba8(),
                    )*
                }
            }

            pub fn format(&self) -> ImageFormat {
                match &self {
                    $(
                        ImageData::$ident(x) => ImageFormat::$ident,
                    )*
                }
            }

            pub fn content_hash(&self) -> u64 {
                match &self {
                    $(
                        ImageData::$ident(x) => x.content_hash(),
                    )*
                }
            }
        }
    }
}

impl_img_data!(
    Luminance4,
    Luminance8,
    LuminanceAlpha4,
    LuminanceAlpha8,
    Rgb5a3,
    Rgb565,
    Rgba8,
    Cmpr
);
