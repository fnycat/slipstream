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
pub struct ItemRoutePoint {
    pub position: glam::Vec3,
    pub bullet_bill_spread: f32,
    pub property1: u16,
    pub property2: u16
}

impl ItemRoutePoint {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let position = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let bullet_bill_spread = reader.read_f32::<BigEndian>()?;
        let property1 = reader.read_u16::<BigEndian>()?;
        let property2 = reader.read_u16::<BigEndian>()?;

        Ok(Self {
            position,
            bullet_bill_spread,
            property1,
            property2
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct ItemRoutePoints {
    pub points: Vec<ItemRoutePoint>
}

impl ItemRoutePoints {
    pub const NAME: &str = "ITPT";

    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        header: KmpSectionHeader
    ) -> SlipstreamResult<Self> {
        // We cap the capacity at 255, as files generally shouldn't have more entries
        // and 255 is still a reasonable upper bound.
        let mut points =
            Vec::with_capacity(header.entry_count.min(MAX_KMP_SECTION_ENTRY_COUNT) as usize);

        for _ in 0..header.entry_count {
            points.push(ItemRoutePoint::deserialize(reader)?);
        }

        Ok(Self { points })
    }
}

impl Visitable for ItemRoutePoints {
    fn accept(&self, node: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_item_route_points(VisitorCtx::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_item_route_points_mut(VisitorCtxMut::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}
