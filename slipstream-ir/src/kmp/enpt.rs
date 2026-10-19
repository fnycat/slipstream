use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::Inspect;
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::encoding::ReadArrayExt;
use crate::kmp::{KmpSectionHeader, MAX_KMP_SECTION_ENTRY_COUNT};
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct EnemyRoutePoint {
    pub position: glam::Vec3,
    pub spread: f32,
    pub point1: u16,
    pub point2: u8,
    pub point3: u8
}

impl EnemyRoutePoint {
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

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct EnemyRoutePoints {
    pub points: Vec<EnemyRoutePoint>
}

impl EnemyRoutePoints {
    pub const NAME: &str = "ENPT";

    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        header: KmpSectionHeader
    ) -> SlipstreamResult<Self> {
        // We cap the capacity at 255, as files generally shouldn't have more entries
        // and 255 is still a reasonable upper bound.
        let mut points =
            Vec::with_capacity(header.entry_count.min(MAX_KMP_SECTION_ENTRY_COUNT) as usize);

        for _ in 0..header.entry_count {
            points.push(EnemyRoutePoint::deserialize(reader)?);
        }

        Ok(Self { points })
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
