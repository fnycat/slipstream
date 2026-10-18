use std::ops::ControlFlow;

use crate::brres::{DeserializeContents, GxBytecode};
use crate::encoding::ReadArrayExt;

use crate::node::node::IrNodeType;
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};
use byteorder::{BigEndian, ReadBytesExt};
use slipstream_shared::cursor::RefCursor;
use slipstream_shared::error::SlipstreamResult;

#[derive(Debug, Clone, PartialEq)]
pub struct Tev {
    pub tex_scales: [u8; 8],
    pub bytecode: GxBytecode
}

impl Visitable for Tev {
    fn accept(&self, node: VisitorCtxMeta<'_>, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_mdl()
            .map(|vis| vis.visit_tev(VisitorCtx::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut<'_>,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_mdl()
            .map(|vis| vis.visit_tev_mut(VisitorCtxMut::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}

impl DeserializeContents for Tev {
    const NAME: &str = "Shaders";
    const KIND: IrNodeType = IrNodeType::Tevs;

    #[tracing::instrument(skip_all)]
    fn deserialize_contents(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let _length = reader.read_u32::<BigEndian>()?;
        let _mdl_offset = reader.read_i32::<BigEndian>()?;
        let _index = reader.read_i32::<BigEndian>()?;
        let _stage_count = reader.read_u8()?;

        reader.set_position(reader.position() + 3); // padding

        let tex_scales = reader.read_u8_array::<8>()?;

        reader.set_position(reader.position() + 8); // padding

        let bytecode = GxBytecode::deserialize_tev_data(reader)?;

        Ok(Self {
            tex_scales,
            bytecode
        })
    }
}
