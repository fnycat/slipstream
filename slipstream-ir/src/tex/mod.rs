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

use crate::brres::{SubfileHeader, SubfileType};
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
    pub header: TexHeader,
    #[inspect(rename = "Image")]
    #[inspect(with = ImageData::draw_ui, read_only)]
    pub data: ImageData
}

impl Texture {
    pub fn serialize(
        &self,
        writer: &mut MutCursor,
        header: &SubfileHeader
    ) -> SlipstreamResult<()> {
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

#[derive(Debug, Clone, PartialEq, Inspect)]
#[inspect(rename = "Texture Header")]
pub struct TexHeader {
    #[inspect(rename = "BRRES Header")]
    pub brres_header: SubfileHeader,
    pub dimensions: glam::U16Vec2,
    pub format: ImageFormat,
    pub mipmaps: u32,
    pub min_mipmap_used: f32,
    pub max_mipmap_used: f32
}

impl TexHeader {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let brres_header = SubfileHeader::deserialize(reader, SubfileType::Tex)?;

        let flags = TextureFlags::deserialize(reader)?;
        let pixel_width = reader.read_u16::<BigEndian>()?;
        let pixel_height = reader.read_u16::<BigEndian>()?;
        let format = ImageFormat::deserialize(reader)?;
        let mipmaps = reader.read_u32::<BigEndian>()?;
        let min_mipmap_used = reader.read_f32::<BigEndian>()?;
        let max_mipmap_used = reader.read_f32::<BigEndian>()?;
        let _unused = reader.read_u32::<BigEndian>()?;

        let dimensions = glam::u16vec2(pixel_width, pixel_height);

        Ok(Self {
            brres_header,
            dimensions,
            format,
            mipmaps,
            min_mipmap_used,
            max_mipmap_used
        })
    }
}

#[tracing::instrument(skip_all, fields(name, parent_id))]
pub fn deserialize(
    reader: &mut RefCursor<[u8]>,
    parent_id: Option<IrNodeKey>,
    arena: &IrArena,
    name: String
) -> SlipstreamResult<IrNodeKey> {
    let header = TexHeader::deserialize(reader)?;
    tracing::debug!(
        "Found TEX0 texture `{name}` of size {}x{} and format `{:?}` with {} mipmaps",
        header.dimensions.x,
        header.dimensions.y,
        header.format,
        header.mipmaps
    );

    if header.dimensions.x > MAX_SAFE_PIXEL_WIDTH as u16 {
        return Err(InvalidInputError {
            reason: format!(
                "image exceeds maximum width of {MAX_SAFE_PIXEL_WIDTH} ({})",
                header.dimensions.x
            ),
            location: Some(reader.position())
        }
        .into());
    }

    if header.dimensions.y > MAX_SAFE_PIXEL_HEIGHT as u16 {
        return Err(InvalidInputError {
            reason: format!(
                "image exceeds maximum height of {MAX_SAFE_PIXEL_HEIGHT} ({})",
                header.dimensions.y
            ),
            location: Some(reader.position())
        }
        .into());
    }

    reader.set_position(header.brres_header.get_section_start(0)?);

    let data = ImageData::deserialize(reader, header.dimensions, header.mipmaps, header.format)?;

    let key = arena.insert(IrNodeDescriptor {
        label: name,
        ty: IrNodeType::Texture,
        parent: parent_id,
        children: Vec::new(),
        contents: ContentSlot::eager(Box::new(Texture { header, data }))
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
