use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::{Inspect, try_from};
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::encoding::ReadArrayExt;
use crate::kmp::{KmpSectionHeader, MAX_KMP_SECTION_ENTRY_COUNT};
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct CheckpointGroup {
    pub first: u8,
    pub count: u8,
    pub previous: [u8; 6],
    pub next: [u8; 6]
}

impl CheckpointGroup {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let first = reader.read_u8()?;
        let count = reader.read_u8()?;
        let previous = reader.read_u8_array::<6>()?;
        let next = reader.read_u8_array::<6>()?;
        let _padding = reader.read_u16::<BigEndian>()?;

        Ok(Self {
            first,
            count,
            previous,
            next
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct CheckpointGroups {
    pub groups: Vec<CheckpointGroup>
}

impl CheckpointGroups {
    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        header: KmpSectionHeader
    ) -> SlipstreamResult<Self> {
        // We cap the capacity at 255, as files generally shouldn't have more entries
        // and 255 is still a reasonable upper bound.
        let mut groups =
            Vec::with_capacity(header.entry_count.min(MAX_KMP_SECTION_ENTRY_COUNT) as usize);

        for _ in 0..header.entry_count {
            groups.push(CheckpointGroup::deserialize(reader)?);
        }

        Ok(Self { groups })
    }
}

impl Visitable for CheckpointGroups {
    fn accept(&self, meta: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_checkpoint_groups(VisitorCtx::new(meta, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        meta: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_checkpoint_groups_mut(VisitorCtxMut::new(meta, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}
