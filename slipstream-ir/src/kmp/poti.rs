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
pub struct PointInformation {
    pub position: glam::Vec3,
    pub settings: u16,
    pub additional_settings: u16
}

impl PointInformation {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let position = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let settings = reader.read_u16::<BigEndian>()?;
        let additional_settings = reader.read_u16::<BigEndian>()?;

        Ok(Self {
            position,
            settings,
            additional_settings
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct PointInformationPath {
    pub enable_smoothing: bool,
    pub cyclic: bool,
    pub points: Vec<PointInformation>
}

impl PointInformationPath {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        const SAFE_POINT_LIMIT: u16 = 1000;

        let point_count = reader.read_u16::<BigEndian>()?;
        let enable_smoothing = reader.read_u8()? != 0;
        let cyclic = reader.read_u8()? != 0;

        if point_count > SAFE_POINT_LIMIT {
            tracing::warn!(
                "KMP POTI entry has a suspiciously large amount of declared points: {point_count}"
            );
        }

        let mut points = Vec::with_capacity(point_count.min(SAFE_POINT_LIMIT) as usize);
        for _ in 0..point_count {
            points.push(PointInformation::deserialize(reader)?);
        }

        Ok(Self {
            enable_smoothing,
            cyclic,
            points
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct PointInformationPaths {
    pub paths: Vec<PointInformationPath>
}

impl PointInformationPaths {
    pub const NAME: &str = "POTI";

    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        header: KmpSectionHeader
    ) -> SlipstreamResult<Self> {
        // We cap the capacity at 255, as files generally shouldn't have more entries
        // and 255 is still a reasonable upper bound.
        let mut paths =
            Vec::with_capacity(header.entry_count.min(MAX_KMP_SECTION_ENTRY_COUNT) as usize);

        for _ in 0..header.entry_count {
            paths.push(PointInformationPath::deserialize(reader)?);
        }

        Ok(Self { paths })
    }
}

impl Visitable for PointInformationPaths {
    fn accept(&self, node: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_point_information(VisitorCtx::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_point_information_mut(VisitorCtxMut::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}
