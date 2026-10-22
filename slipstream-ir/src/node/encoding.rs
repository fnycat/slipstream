use std::ops::ControlFlow;

use slipstream_shared::SlipstreamResult;
use slipstream_shared::cursor::MutCursor;
use slipstream_shared::error::UnsupportedError;

use crate::arc::{self, ArcDirectory};
use crate::brres::{
    ColorBuffer, Definitions, DeserializeContents, Material, MdlSectionType, NormalBuffer,
    PaletteLinks, Polygon, Tev, TextureLinks, UvBuffer, VertexBuffer
};
use crate::fixup::FixupPass;
use crate::node::arena::IrArena;
use crate::node::lazy::{DeferPayload, DynContent};
use crate::node::node::{IrNode, IrNodeType};
use crate::visitor::{Visitor, VisitorCtx};

pub struct SerializeVisitor<'a> {
    arena: &'a IrArena,
    pass: &'a mut FixupPass,
    writer: &'a mut MutCursor,
    result: SlipstreamResult<()>
}

impl Visitor for SerializeVisitor<'_> {
    fn visit_arc(&mut self, arc: VisitorCtx<'_, ArcDirectory>) -> ControlFlow<()> {
        // Only serialize the root ARC directory. It will serialize all its children with it.
        match arc.meta.ty {
            IrNodeType::ArcDirectory { is_root: true, .. } => {
                self.result = arc::serialize(self.arena, arc.meta.key, self.writer, self.pass);
                match self.result {
                    Ok(()) => ControlFlow::Continue(()),
                    Err(_) => ControlFlow::Break(())
                }
            }
            IrNodeType::ArcDirectory { is_root: false, .. } => ControlFlow::Continue(()),
            _ => unreachable!("called `visit_arc` on file that is not of type ARC")
        }
    }
}

/// Serializes the given node and all its children to the given writer.
pub fn serialize_node(
    arena: &IrArena,
    node: &IrNode,
    writer: &mut MutCursor
) -> SlipstreamResult<()> {
    tracing::trace!("Serializing node {:?}", node.key());

    let mut pass = FixupPass::new();
    let mut visitor = SerializeVisitor {
        arena,
        writer,
        pass: &mut pass,
        result: Ok(())
    };
    arena.visit_all(node.key, &mut visitor)?;
    visitor.result
}

#[cold]
pub fn deserialize_node(mut payload: DeferPayload) -> SlipstreamResult<Box<DynContent>> {
    Ok(match payload.ty {
        IrNodeType::MdlSection { ty } => match ty {
            MdlSectionType::Definitions => {
                Box::new(Definitions::deserialize_contents(&mut payload.reader)?)
            }
            MdlSectionType::Vertices => {
                Box::new(VertexBuffer::deserialize_contents(&mut payload.reader)?)
            }
            MdlSectionType::Normals => {
                Box::new(NormalBuffer::deserialize_contents(&mut payload.reader)?)
            }
            MdlSectionType::Colors => {
                Box::new(ColorBuffer::deserialize_contents(&mut payload.reader)?)
            }
            MdlSectionType::UvCoordinates => {
                Box::new(UvBuffer::deserialize_contents(&mut payload.reader)?)
            }
            MdlSectionType::Material => {
                Box::new(Material::deserialize_contents(&mut payload.reader)?)
            }
            MdlSectionType::Shader => Box::new(Tev::deserialize_contents(&mut payload.reader)?),
            MdlSectionType::Polygon => {
                Box::new(Polygon::deserialize_contents(&mut payload.reader)?)
            }
            MdlSectionType::TextureLinks => {
                Box::new(TextureLinks::deserialize_contents(&mut payload.reader)?)
            }
            MdlSectionType::PaletteLinks => {
                Box::new(PaletteLinks::deserialize_contents(&mut payload.reader)?)
            }
            _ => {
                return Err(UnsupportedError {
                    reason: format!(
                        "lazily parsing a node of MDL section {:?} is not supported",
                        ty
                    ),
                    location: Some(payload.reader.position())
                }
                .into());
            }
        },
        _ => {
            return Err(UnsupportedError {
                reason: format!(
                    "lazily parsing a node of type {:?} is not supported",
                    payload.ty
                ),
                location: Some(payload.reader.position())
            }
            .into());
        }
    })
}
