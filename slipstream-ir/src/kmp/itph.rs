use std::ops::ControlFlow;

use byteorder::ReadBytesExt;
use slipstream_derive::Inspect;
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::encoding::ReadArrayExt;
use crate::kmp::{KmpSectionHeader, MAX_KMP_SECTION_ENTRY_COUNT};
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct ItemRoute {
    pub point_start: u8,
    pub point_length: u8,
    pub previous_group: [i8; 6],
    pub next_group: [i8; 6]
}

impl ItemRoute {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let point_start = reader.read_u8()?;
        let point_length = reader.read_u8()?;
        let previous_group = reader.read_i8_array::<6>()?;
        let next_group = reader.read_i8_array::<6>()?;

        Ok(Self {
            point_start,
            point_length,
            previous_group,
            next_group
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct ItemRoutes {
    pub routes: Vec<ItemRoute>
}

impl ItemRoutes {
    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        header: KmpSectionHeader
    ) -> SlipstreamResult<Self> {
        // We cap the capacity at 255, as files generally shouldn't have more entries
        // and 255 is still a reasonable upper bound.
        let mut routes =
            Vec::with_capacity(header.entry_count.min(MAX_KMP_SECTION_ENTRY_COUNT) as usize);

        for _ in 0..header.entry_count {
            routes.push(ItemRoute::deserialize(reader)?);
        }

        Ok(Self { routes })
    }
}

impl Visitable for ItemRoutes {
    fn accept(&self, node: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_item_routes(VisitorCtx::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_item_routes_mut(VisitorCtxMut::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}
