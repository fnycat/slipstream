use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::Inspect;
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::encoding::ReadArrayExt;
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct EnemyRoutePoints {
    pub position: glam::Vec3,
    pub spread: f32,
    pub point1: u16,
    pub point2: u8,
    pub point3: u8
}

impl EnemyRoutePoints {
    pub const NAME: &str = "ENPT";

    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let position = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let spread = reader.read_f32::<BigEndian>()?;
        let point1 = reader.read_u16::<BigEndian>()?;
        let point2 = reader.read_u8()?;
        let point3 = reader.read_u8()?;

        Ok(Self {
            position,
            spread,
            point1,
            point2,
            point3
        })
    }
}

impl Visitable for EnemyRoutePoints {
    fn accept(&self, node: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_enemy_route_points(VisitorCtx::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_enemy_route_points_mut(VisitorCtxMut::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}
