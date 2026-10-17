use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::{Inspect, try_from};
use slipstream_shared::error::{CorruptionError, IncorrectFormat};
use slipstream_shared::{RefCursor, SlipstreamError, SlipstreamResult};

use crate::encoding::{Deserialize, ReadArrayExt};
use crate::mdl::{MagnificationFilter, MinificationFilter, WrapMode};
use crate::node::arena::{IrArena, IrNodeDescriptor, IrNodeKey};
use crate::node::node::{ContentSlot, IrNodeType};
use crate::tpl::{ImageData, ImageFormat};
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

pub const TPL_MAGIC: [u8; 4] = [0x00, 0x20, 0xAF, 0x30];

#[derive(Debug, Clone, PartialEq, Eq, Inspect)]
#[inspect(summary = "TplHeader::summary")]
pub struct TplHeader {
    #[inspect(tooltip = "Cursor position at the start of the header.")]
    pub header_start: u64,
    #[inspect(tooltip = "Number of images in this file.")]
    pub image_count: u32,
    #[inspect(tooltip = "Offset in bytes to the image table.")]
    pub table_offset: i32
}

impl TplHeader {
    #[inline]
    pub fn summary(&self) -> String {
        format!("TPL Header: {} image(s)", self.image_count)
    }

    #[inline]
    pub fn get_table_start(&self) -> u64 {
        (self.header_start as i64 + self.table_offset as i64) as u64
    }

    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let header_start = reader.position();

        let magic = reader.read_u8_array::<4>()?;
        if magic != TPL_MAGIC {
            return Err(IncorrectFormat {
                expected_magic: TPL_MAGIC.into(),
                found_magic: magic.into(),
                location: Some(reader.position())
            }
            .into());
        }

        let image_count = reader.read_u32::<BigEndian>()?;
        let table_offset = reader.read_i32::<BigEndian>()?;

        Ok(Self {
            header_start,
            image_count,
            table_offset
        })
    }
}

impl Visitable for TplHeader {
    fn accept(&self, node: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor.visit_tpl(VisitorCtx::new(node, self))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor.visit_tpl_mut(VisitorCtxMut::new(node, self))
    }
}

#[try_from(u32)]
#[derive(Debug, Clone, PartialEq, Inspect)]
pub enum PaletteFormat {
    #[inspect(rename = "8-Bit Luminance + Alpha")]
    La8 = 0x00,
    #[inspect(rename = "RGB565")]
    Rgb565 = 0x01,
    #[inspect(rename = "RGB5A3")]
    Rgba5a3 = 0x02
}

impl PaletteFormat {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaletteHeader {}

impl PaletteHeader {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let entry_count = reader.read_u16::<BigEndian>()?;
        let unpacked = reader.read_u8()?;
        let _padding = reader.read_u8()?;
        let palette_format = PaletteFormat::deserialize(reader)?;

        todo!()
    }
}

pub const UNUSUAL_TPL_IMAGE_COUNT: u32 = 100;

pub fn deserialize(
    reader: &mut RefCursor<[u8]>,
    parent: Option<IrNodeKey>,
    arena: &IrArena,
    label: String
) -> SlipstreamResult<IrNodeKey> {
    let tpl_start = reader.position();

    let header = TplHeader::deserialize(reader)?;
    reader.set_position(header.get_table_start());

    let mut next_block_start = reader.position();

    let root_key = arena.reserve_key();

    // All known TPL files have either 1 or 2 images. So we just cap the capacity
    // at something like 10. If a user-made TPL file has more images, the capacity will grow by itself.

    if header.image_count > UNUSUAL_TPL_IMAGE_COUNT {
        tracing::error!(
            "The TPL file has an unusually large amount of declared images: {}",
            header.image_count
        );
    }

    let mut images = Vec::with_capacity(header.image_count.min(10) as usize);
    for i in 0..header.image_count {
        reader.set_position(next_block_start);
        let image_offset = reader.read_i32::<BigEndian>()?;
        let palette_offset = reader.read_i32::<BigEndian>()?;
        next_block_start = reader.position();

        let image_header_start = (tpl_start as i64).checked_add(image_offset as i64).ok_or_else(|| {
            SlipstreamError::from(CorruptionError {
                reason: format!("TPL image header offset exceeds 64-bit integer limit: (offset: {image_offset})"),
                ..Default::default()
            })
        })?;

        reader.set_position(image_header_start as u64);
        let image_header = ImageHeader::deserialize(reader)?;

        let mipmaps_used = image_header.max_lod.saturating_sub(image_header.min_lod) + 1;

        reader.set_position(image_header.get_data_start(tpl_start)?);
        let image = ImageData::deserialize(
            reader,
            image_header.dimensions,
            mipmaps_used as u32,
            image_header.format
        )?;

        let img_key = arena.insert(IrNodeDescriptor {
            label: format!("Image #{}", i + 1),
            ty: IrNodeType::TplImage,
            parent: Some(root_key),
            contents: ContentSlot::eager(Box::new(TplImage {
                header: image_header,
                image
            })),
            ..Default::default()
        });
        images.push(img_key);
    }

    arena.insert_at(
        root_key,
        IrNodeDescriptor {
            label,
            ty: IrNodeType::Tpl,
            parent,
            children: images,
            contents: ContentSlot::eager(Box::new(header))
        }
    );

    Ok(root_key)
}

#[derive(Debug, Clone, PartialEq, Inspect)]
#[inspect(rename = "Image Header")]
pub struct ImageHeader {
    pub dimensions: glam::U16Vec2,
    pub format: ImageFormat,
    /// Offset of the image data, relative to the start of the TPL file.
    pub image_offset: u32,
    pub wrap_s: WrapMode,
    pub wrap_t: WrapMode,
    #[inspect(rename = "Minification Filter")]
    pub min_filter: MinificationFilter,
    #[inspect(rename = "Magnification Filter")]
    pub mag_filter: MagnificationFilter,
    #[inspect(rename = "LOD Bias")]
    pub lod_bias: f32,
    #[inspect(rename = "Enable Edge LOD")]
    pub enable_edge_lod: bool,
    #[inspect(rename = "Minimum LOD")]
    pub min_lod: u8,
    #[inspect(rename = "Maximum LOD")]
    pub max_lod: u8,
    pub unpacked: u8
}

impl ImageHeader {
    /// Computes the location of the image data based on the given TPL file start location.
    pub fn get_data_start(&self, tpl_start: u64) -> SlipstreamResult<u64> {
        tpl_start
            .checked_add(self.image_offset as u64)
            .ok_or_else(|| {
                CorruptionError {
                    reason: format!(
                        "image data location exceeds 64-bit integer limit (offset: {})",
                        self.image_offset
                    ),
                    ..Default::default()
                }
                .into()
            })
    }

    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let height = reader.read_u16::<BigEndian>()?;
        let width = reader.read_u16::<BigEndian>()?;
        let dimensions = glam::u16vec2(width, height);

        let format = ImageFormat::deserialize(reader)?;
        let image_data_address = reader.read_u32::<BigEndian>()?;
        let wrap_s = WrapMode::deserialize(reader)?;
        let wrap_t = WrapMode::deserialize(reader)?;

        let min_filter = MinificationFilter::deserialize(reader)?;
        let mag_filter = MagnificationFilter::deserialize(reader)?;

        let lod_bias = reader.read_f32::<BigEndian>()?;
        let enable_edge_lod = reader.read_u8()? != 0;
        let min_lod = reader.read_u8()?;
        let max_lod = reader.read_u8()?;
        let unpacked = reader.read_u8()?;

        Ok(Self {
            dimensions,
            format,
            image_offset: image_data_address,
            wrap_s,
            wrap_t,
            min_filter,
            mag_filter,
            lod_bias,
            enable_edge_lod,
            min_lod,
            max_lod,
            unpacked
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
#[inspect(rename = "TPL Image")]
pub struct TplImage {
    pub header: ImageHeader,
    #[inspect(with = ImageData::draw_ui, read_only)]
    pub image: ImageData
}

impl Visitable for TplImage {
    fn accept(&self, node: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor.visit_tpl_image(VisitorCtx::new(node, self))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor.visit_tpl_image_mut(VisitorCtxMut::new(node, self))
    }
}
