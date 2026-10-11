use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::{Inspect, try_from};
use slipstream_shared::error::UnsupportedError;
use slipstream_shared::{RefCursor, SlipstreamResult, try_unwrap};

use crate::brres::{BFileHeader, BFileType};
use crate::encoding::Deserialize;
use crate::img::{CmprDescriptor, CmprImage};
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

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Texture {
    #[inspect(tooltip = "Width and height of the texture in pixels")]
    pub size: glam::U16Vec2,
    pub format: TextureFormat,
    pub mipmap_count: u32,
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

    match format {
        TextureFormat::Cmpr => {
            let cmpr = CmprImage::deserialize(
                reader,
                CmprDescriptor {
                    width: pixel_width,
                    height: pixel_height,
                    mipmap_count,
                },
            )?;
        }
        _ => {}
    }

    let key = arena.insert(IrNodeDescriptor {
        label: name,
        ty: IrNodeType::Texture,
        parent: Some(parent_id),
        children: Vec::new(),
        contents: ContentSlot::eager(Box::new(Texture {
            size: glam::u16vec2(pixel_width, pixel_height),
            format,
            mipmap_count,
        })),
    });

    Ok(key)
}
