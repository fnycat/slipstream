use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::Inspect;
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::encoding::ReadArrayExt;
use crate::kmp::{KmpSectionHeader, MAX_KMP_SECTION_ENTRY_COUNT};
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

/// Special version of `Box2` to change the names of the fields.
#[derive(Debug, Copy, Clone, PartialEq, Inspect, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct CheckpointBox2 {
    #[inspect(rename = "Left Edge")]
    pub left: glam::Vec2,
    #[inspect(rename = "Right Edge")]
    pub right: glam::Vec2
}

impl CheckpointBox2 {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let left = glam::Vec2::from_array(reader.read_f32_array::<2, BigEndian>()?);
        let right = glam::Vec2::from_array(reader.read_f32_array::<2, BigEndian>()?);

        Ok(Self { left, right })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Checkpoint {
    pub bounds: CheckpointBox2,
    /// Index into the `JGPT` section to respawn the player at once they've entered
    /// this checkpoint.
    pub respawn_index: u8,
    /// Checkpoint type. `0x00` for the lap counter, `0x01-0xFE` for key checkpoints
    /// and `0xFF` for a regular checkpoint.
    pub ty: i8,
    /// Previous checkpoint in the group's sequence. `0xFF` for the first point of the group.
    pub previous: u8,
    /// Next checkpoint in the group's sequence. `0xFF` for the last point of the group.
    pub next: u8
}

impl Checkpoint {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let bounds = CheckpointBox2::deserialize(reader)?;
        let respawn_index = reader.read_u8()?;
        let ty = reader.read_i8()?;
        let previous = reader.read_u8()?;
        let next = reader.read_u8()?;

        Ok(Self {
            bounds,
            respawn_index,
            ty,
            previous,
            next
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Checkpoints {
    pub checkpoints: Vec<Checkpoint>
}

impl Checkpoints {
    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        header: KmpSectionHeader
    ) -> SlipstreamResult<Self> {
        // We cap the capacity at 255, as files generally shouldn't have more entries
        // and 255 is still a reasonable upper bound.
        let mut checkpoints =
            Vec::with_capacity(header.entry_count.min(MAX_KMP_SECTION_ENTRY_COUNT) as usize);

        for _ in 0..header.entry_count {
            checkpoints.push(Checkpoint::deserialize(reader)?);
        }

        Ok(Self { checkpoints })
    }
}

impl Visitable for Checkpoints {
    fn accept(&self, meta: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_checkpoints(VisitorCtx::new(meta, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        meta: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_checkpoints_mut(VisitorCtxMut::new(meta, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}
