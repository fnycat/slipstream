use slipstream_shared::{SlipstreamResult, cursor::MutCursor, error::UnsupportedError};

use crate::{
    mdl0::{
        ColorBuffer, Definitions, DeserializeContents, MaterialBuffer, NormalBuffer, PaletteLinks,
        Polygon, Tev, TextureLinks, UvBuffer, VertexBuffer,
    },
    node::{
        lazy::{DeferPayload, DynContent},
        node::{IrNode, IrNodeType},
    },
};

pub fn serialize_node(node: &IrNode, writer: &mut MutCursor) -> SlipstreamResult<()> {
    todo!()
}

#[cold]
pub fn deserialize_node(mut payload: DeferPayload) -> SlipstreamResult<Box<DynContent>> {
    Ok(match payload.ty {
        IrNodeType::Definitions => {
            Box::new(Definitions::deserialize_contents(&mut payload.reader)?)
        }
        IrNodeType::VertexBuffer => {
            Box::new(VertexBuffer::deserialize_contents(&mut payload.reader)?)
        }
        IrNodeType::NormalBuffer => {
            Box::new(NormalBuffer::deserialize_contents(&mut payload.reader)?)
        }
        IrNodeType::ColorBuffer => {
            Box::new(ColorBuffer::deserialize_contents(&mut payload.reader)?)
        }
        IrNodeType::UvBuffer => Box::new(UvBuffer::deserialize_contents(&mut payload.reader)?),
        IrNodeType::Material => {
            Box::new(MaterialBuffer::deserialize_contents(&mut payload.reader)?)
        }
        IrNodeType::Tevs => Box::new(Tev::deserialize_contents(&mut payload.reader)?),
        IrNodeType::Polygon => Box::new(Polygon::deserialize_contents(&mut payload.reader)?),
        IrNodeType::TextureLinks => {
            Box::new(TextureLinks::deserialize_contents(&mut payload.reader)?)
        }
        IrNodeType::PaletteLinks => {
            Box::new(PaletteLinks::deserialize_contents(&mut payload.reader)?)
        }
        _ => {
            return Err(UnsupportedError {
                reason: format!(
                    "lazily parsing a node of type {:?} is not supported",
                    payload.ty
                ),
                location: Some(payload.reader.position()),
            }
            .into());
        }
    })
}
