use std::ops::ControlFlow;

use slipstream_shared::{SlipstreamResult, cursor::MutCursor, error::UnsupportedError};

use crate::{
    arc::{self, ArcDirectory},
    deferred_pass::DeferredPass,
    mdl0::{
        ColorBuffer, Definitions, DeserializeContents, MaterialBuffer, NormalBuffer, PaletteLinks,
        Polygon, Tev, TextureLinks, UvBuffer, VertexBuffer,
    },
    node::{
        arena::IrArena,
        lazy::{DeferPayload, DynContent},
        node::{IrNode, IrNodeType},
    },
    visitor::{Visitor, VisitorContext},
};

pub struct SerializeVisitor<'a> {
    arena: &'a IrArena,
    pass: &'a mut DeferredPass,
    writer: &'a mut MutCursor,
    result: SlipstreamResult<()>,
}

impl Visitor for SerializeVisitor<'_> {
    fn visit_arc(&mut self, arc: VisitorContext<'_, ArcDirectory>) -> ControlFlow<()> {
        self.result = arc::serialize(self.arena, arc.meta.key, self.writer, self.pass);
        match self.result {
            Ok(()) => ControlFlow::Continue(()),
            Err(_) => ControlFlow::Break(()),
        }
    }
}

/// Serializes the given node and all its children to the given writer.
pub fn serialize_node(
    arena: &IrArena,
    node: &IrNode,
    writer: &mut MutCursor,
) -> SlipstreamResult<()> {
    tracing::trace!("Serializing node {:?}", node.key());

    let mut pass = DeferredPass::new();
    let mut visitor = SerializeVisitor {
        arena,
        writer,
        pass: &mut pass,
        result: Ok(()),
    };
    arena.walk(node.key, &mut visitor)?;
    visitor.result
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
