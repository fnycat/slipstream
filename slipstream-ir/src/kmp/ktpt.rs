use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::Inspect;
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::encoding::ReadArrayExt;
use crate::kmp::{KmpSectionHeader, MAX_KMP_SECTION_ENTRY_COUNT};
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

pub const STARTING_POSITION_COUNT: usize = 12;

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct StartingPosition {
    pub position: glam::Vec3,
    pub rotation: glam::Vec3,
    pub player: i16
}

impl StartingPosition {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let position = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let rotation = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let player = reader.read_i16::<BigEndian>()?;
        let _padding = reader.read_u16::<BigEndian>()?;

        Ok(Self {
            position,
            rotation,
            player
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct StartingPositions {
    #[inspect(max_horizontal_elements = 1)]
    pub players: Vec<StartingPosition>
}

impl StartingPositions {
    pub const NAME: &str = "KTPT";

    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        header: KmpSectionHeader
    ) -> SlipstreamResult<Self> {
        // We cap the capacity at 255, as files generally shouldn't have more entries
        // and 255 is still a reasonable upper bound.
        //
        // This should generally just have 12 entries, but idk maybe some custom tracks have
        // more.
        let mut players =
            Vec::with_capacity(header.entry_count.min(MAX_KMP_SECTION_ENTRY_COUNT) as usize);

        for _ in 0..STARTING_POSITION_COUNT {
            players.push(StartingPosition::deserialize(reader)?);
        }

        Ok(Self { players })
    }
}

impl Visitable for StartingPositions {
    fn accept(&self, node: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_starting_positions(VisitorCtx::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_starting_positions_mut(VisitorCtxMut::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}
