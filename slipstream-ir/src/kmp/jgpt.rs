use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
use glam::i16;
use slipstream_derive::Inspect;
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::encoding::ReadArrayExt;
use crate::kmp::{KmpSectionHeader, MAX_KMP_SECTION_ENTRY_COUNT};
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct RespawnPoint {
    pub position: glam::Vec3,
    pub rotation: glam::Vec3,
    pub id: u16,
    pub user_data: i16
}

impl RespawnPoint {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let position = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let rotation = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let id = reader.read_u16::<BigEndian>()?;
        let user_data = reader.read_i16::<BigEndian>()?;

        Ok(Self {
            position, rotation, id, user_data
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Respawns {
    pub respawns: Vec<RespawnPoint>
}

impl Respawns {
    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        header: KmpSectionHeader
    ) -> SlipstreamResult<Self> {
        // We cap the capacity at 255, as files generally shouldn't have more entries
        // and 255 is still a reasonable upper bound.
        let mut respawns =
            Vec::with_capacity(header.entry_count.min(MAX_KMP_SECTION_ENTRY_COUNT) as usize);

        for _ in 0..header.entry_count {
            respawns.push(RespawnPoint::deserialize(reader)?);
        }

        Ok(Self { respawns })
    }
}

impl Visitable for Respawns {
    fn accept(&self, meta: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_respawns(VisitorCtx::new(meta, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        meta: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_respawns_mut(VisitorCtxMut::new(meta, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}
