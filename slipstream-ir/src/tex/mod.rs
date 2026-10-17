use std::io::{BufWriter, Seek, Write};
use std::ops::ControlFlow;
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
#[cfg(feature = "egui-impls")]
use eframe::egui_wgpu;
use image::{ImageBuffer, Rgba};
use slipstream_derive::{Inspect, inspect_bitfield, try_from};
use slipstream_shared::cursor::MutCursor;
use slipstream_shared::error::{InvalidInputError, UnsupportedError};
use slipstream_shared::inspect::Changes;
#[cfg(feature = "egui-impls")]
use slipstream_shared::inspect::FieldConfig;
use slipstream_shared::{RefCursor, SlipstreamError, SlipstreamResult, try_unwrap};

use crate::brres::{BFileHeader, BFileType};
use crate::encoding::{Deserialize, WriteArrayExt};
use crate::mdl::UvDataType::S;
use crate::node::arena::{IrArena, IrNodeDescriptor, IrNodeKey};
use crate::node::node::{ContentSlot, IrNodeType};
use crate::tpl::{
    CmprImage, EncodedImage, ExportableImage, ImageData, ImageFormat, L4Image, L8Image, La4Image,
    La8Image, MAX_SAFE_PIXEL_HEIGHT, MAX_SAFE_PIXEL_WIDTH, Rgb5a3Image, Rgb565Image, Rgba8Image
};
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

pub const TEX_MAGIC: [u8; 4] = [0x54, 0x45, 0x58, 0x30]; // "TEX"

#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct TextureFlags {
    pub is_paletted: bool,
    #[bits(31)]
    _unused: u32
}

impl TextureFlags {
    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(self.into_bits())?;
        Ok(())
    }

    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Texture {
    #[inspect(tooltip = "Width and height of the texture in pixels")]
    pub size: glam::U16Vec2,
    #[inspect(rename = "Image")]
    #[inspect(with = ImageData::draw_ui, read_only)]
    pub data: ImageData
}

impl Texture {
    pub fn serialize(&self, writer: &mut MutCursor, header: &BFileHeader) -> SlipstreamResult<()> {
        header.serialize(writer)?;

        let format = self.data.format();
        let flags = TextureFlags::new().with_is_paletted(format.is_paletted());

        flags.serialize(writer)?;

        let dims = self.data.dimensions();
        writer.write_u16_array::<2, BigEndian>(dims.to_array())?;

        format.serialize(writer)?;
        writer.write_u32::<BigEndian>(self.data.mipmaps())?;

        // Minimum mipmap used. This seems to always be zero.
        writer.write_u32::<BigEndian>(0)?;

        tracing::error!("TODO: DETERMINE MAXIMUM MIPMAP USED");
        writer.write_u32::<BigEndian>(1)?;

        // Offset to absolute path of intermediate file. Unused.
        writer.write_u32::<BigEndian>(0)?;

        Ok(())
    }
}

impl Visitable for Texture {
    fn accept(&self, node: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor.visit_tex(VisitorCtx::new(node, self))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> std::ops::ControlFlow<()> {
        visitor.visit_tex_mut(VisitorCtxMut::new(node, self))
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

    let flags = TextureFlags::deserialize(reader)?;
    let pixel_width = reader.read_u16::<BigEndian>()?;
    let pixel_height = reader.read_u16::<BigEndian>()?;
    let format = ImageFormat::deserialize(reader)?;
    let mipmaps = reader.read_u32::<BigEndian>()?;
    let min_mipmap_used = reader.read_f32::<BigEndian>()?;
    let max_mipmap_used = reader.read_f32::<BigEndian>()?;
    let _unused = reader.read_u32::<BigEndian>()?;

    if pixel_width > MAX_SAFE_PIXEL_WIDTH as u16 {
        return Err(InvalidInputError {
            reason: format!(
                "image exceeds maximum width of {MAX_SAFE_PIXEL_WIDTH} ({pixel_width})"
            ),
            location: Some(reader.position())
        }
        .into());
    }

    if pixel_height > MAX_SAFE_PIXEL_HEIGHT as u16 {
        return Err(InvalidInputError {
            reason: format!(
                "image exceeds maximum height of {MAX_SAFE_PIXEL_HEIGHT} ({pixel_height})"
            ),
            location: Some(reader.position())
        }
        .into());
    }

    reader.set_position(subfile_header.get_section_start(0)?);

    let dims = glam::u16vec2(pixel_width, pixel_height);
    let data = ImageData::deserialize(reader, dims, mipmaps, format)?;

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

    use crate::node::arena::IrArena;
    use crate::tex;

    // Found via fuzzing. This corrupted file seemed to cause an impossibly large allocation.
    const OOM_CORPUS: &[u8] = &[
        255, 0, 3, 255, 255, 255, 0, 0, 0, 0, 0, 3, 255, 255, 255, 3, 0, 0, 0, 49, 0, 0, 0, 0, 0,
        0, 84, 250, 255, 251, 255, 1, 0, 0, 0, 3, 255, 255, 255, 255, 255, 5, 3, 87, 138, 255, 255,
        255, 255, 255, 255, 254, 3, 0, 0, 138, 255, 255, 5, 3, 255, 87
    ];

    #[test]
    fn oom_corpus() {
        let mut reader = RefCursor::new(Arc::from(OOM_CORPUS));
        let arena = IrArena::new();
        let _ = tex::deserialize(&mut reader, None, &arena, String::from("oom_corpus"));
    }
}
