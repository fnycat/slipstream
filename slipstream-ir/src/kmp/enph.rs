use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::{Inspect, inspect_bitfield};
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::encoding::ReadArrayExt;
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

#[inspect_bitfield(u16)]
#[derive(PartialEq, Eq)]
pub struct EnemyRouteLinks {
    #[bits(6)]
    pub link_backward: u8,
    #[bits(6)]
    pub link_forward: u8,
    #[bits(4)]
    _unused: u8
}

impl EnemyRouteLinks {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let short = reader.read_u16::<BigEndian>()?;
        Ok(Self::from_bits(short))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Inspect)]
pub struct EnemyRoute {
    pub point_start: u8,
    pub point_length: u8,
    pub previous_group: [u8; 6],
    pub next_group: [u8; 6],
    pub links: EnemyRouteLinks
}

impl EnemyRoute {
    pub const NAME: &str = "ENPH";

    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let point_start = reader.read_u8()?;
        let point_length = reader.read_u8()?;
        let previous_group = reader.read_u8_array::<6>()?;
        let next_group = reader.read_u8_array::<6>()?;
        let links = EnemyRouteLinks::deserialize(reader)?;

        Ok(Self {
            point_start,
            point_length,
            previous_group,
            next_group,
            links
        })
    }
}

impl Visitable for EnemyRoute {
    fn accept(&self, meta: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_enemy_route(VisitorCtx::new(meta, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        meta: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_enemy_route_mut(VisitorCtxMut::new(meta, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}
