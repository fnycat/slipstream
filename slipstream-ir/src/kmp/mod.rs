mod area;
mod ckph;
mod ckpt;
mod enph;
mod enpt;
mod gobj;
mod itph;
mod itpt;
mod ktpt;
mod poti;
mod visitor;

pub use area::*;
pub use ckph::*;
pub use ckpt::*;
pub use enph::*;
pub use enpt::*;
pub use gobj::*;
pub use itph::*;
pub use itpt::*;
pub use ktpt::*;
pub use poti::*;
pub use visitor::*;

use std::ops::ControlFlow;

use slipstream_shared::error::CorruptionError;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::{Inspect, try_from};
use slipstream_shared::{RefCursor, SlipstreamError, SlipstreamResult, verify};

use crate::encoding::ReadArrayExt;
use crate::node::arena::{IrArena, IrNodeDescriptor, IrNodeKey};
use crate::node::node::{ContentSlot, IrNodeType};
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

pub const KMP_MAGIC: [u8; 4] = [0x52, 0x4b, 0x4d, 0x44]; // "RKMD"
pub const DEFAULT_KMP_VERSION: u32 = 0x9d8;
/// The maximum amount of possible KMP files.
pub const MAX_KMP_SECTION_COUNT: u32 = 15;
/// A KMP section cannot have more than 255 entries, otherwise the Wii crashes.
pub const MAX_KMP_SECTION_ENTRY_COUNT: u16 = 255;

pub const KMP_READABLE_SECTION_NAMES: [&str; 15] = [
    "Starting Positions",
    "Enemy Route Points",
    "Enemy Route",
    "Item Route Points",
    "Item Route",
    "Checkpoints",
    "Checkpoint Paths",
    "Geo Objects",
    "Routes",
    "Areas",
    "Cameras",
    "Respawn Positions",
    "Cannon Target Positions",
    "Mission Success Points",
    "Stage Information"
];

/// These are just four letter magics for each of the files, but converted to an `u32` for easier
/// reading.
#[try_from(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum KmpSectionType {
    /// Magic: KTPT
    StartingPositions = 0x4b545054,
    /// Magic: ENPT
    EnemyRoutePoints = 0x454e5054,
    /// Magic: ENPH
    EnemyRoute = 0x454e5048,
    ItemRoutePoints = 0x49545054,
    ItemRoute = 0x49545048,
    Checkpoints = 0x434b5054,
    CheckpointGroups = 0x434b5048,
    GeoObjects = 0x474f424a,
    Routes = 0x504f5449,
    Areas = 0x41524541,
    Cameras = 0x43414d45,
    RespawnPositions = 0x4a475054,
    CannonTargetPositions = 0x434e5054,
    MissionSuccessPoints = 0x4d535054,
    StageInformation = 0x53544749
}

impl KmpSectionType {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct KmpHeader {
    #[inspect(tooltip = "Pointer to the start of the KMP file.")]
    pub start: u32,
    #[inspect(tooltip = "The length in bytes of the entire KMP file.")]
    pub length: u32,
    #[inspect(tooltip = "Offsets to the subsections within the KMP file.")]
    pub offsets: Vec<u32>
}

impl KmpHeader {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let start = reader.position() as u32;

        let magic = reader.read_u8_array::<4>()?;
        verify!(
            magic == KMP_MAGIC,
            "KMP file magic was expected to be `RKMD`, was actually `{}`. This is not a KMP file",
            String::from_utf8_lossy(&magic)
        );

        let length = reader.read_u32::<BigEndian>()?;
        let mut section_number = reader.read_u16::<BigEndian>()?;
        let _header_length = reader.read_u16::<BigEndian>()?;

        let version = reader.read_u32::<BigEndian>()?;
        verify!(
            version == DEFAULT_KMP_VERSION,
            "version {version} is not supported, only {DEFAULT_KMP_VERSION} is"
        );

        if section_number > MAX_KMP_SECTION_COUNT as u16 {
            tracing::debug!(
                "Section number exceeds maximum KMP file count, capping it to {MAX_KMP_SECTION_COUNT}"
            );
            section_number = MAX_KMP_SECTION_COUNT as u16;
        }

        let mut offsets = Vec::with_capacity(section_number as usize);
        for _ in 0..section_number {
            offsets.push(reader.read_u32::<BigEndian>()?);
        }

        Ok(Self {
            start,
            length,
            offsets
        })
    }
}

impl Visitable for KmpHeader {
    fn accept(&self, node: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_kmp(VisitorCtx::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_kmp_mut(VisitorCtxMut::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct KmpSectionHeader {
    pub ty: KmpSectionType,
    pub entry_count: u16,
    pub extra_data: u16
}

impl KmpSectionHeader {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let ty = KmpSectionType::deserialize(reader)?;
        let entry_count = reader.read_u16::<BigEndian>()?;
        let extra_data = reader.read_u16::<BigEndian>()?;

        Ok(Self {
            ty,
            entry_count,
            extra_data
        })
    }
}

pub fn deserialize(
    reader: &mut RefCursor<[u8]>,
    parent: Option<IrNodeKey>,
    arena: &IrArena,
    name: String
) -> SlipstreamResult<IrNodeKey> {
    let header = KmpHeader::deserialize(reader)?;
    let header_end = reader.position();

    let kmp_root = arena.reserve_key();

    let mut children = Vec::with_capacity(header.offsets.len());
    for &offset in &header.offsets {
        // All offsets are relative to the end of the header, not to the start!!
        let data_start = (header_end as i64)
            .checked_add(offset as i64)
            .ok_or_else(|| {
                SlipstreamError::from(CorruptionError {
                    reason: format!(
                        "pointer to KMP section overflows 32-bit integer (offset: {offset})"
                    ),
                    ..Default::default()
                })
            })?;

        reader.set_position(data_start as u64);

        let section_header = KmpSectionHeader::deserialize(reader)?;
        let section_ty = section_header.ty; // copy it because `section_header` moves

        if section_header.entry_count > MAX_KMP_SECTION_ENTRY_COUNT {
            tracing::warn!(
                "KMP section {section_ty:?} has more than {MAX_KMP_SECTION_ENTRY_COUNT} entries, this will crash a real console"
            );
        }

        let content: Box<dyn Visitable + Send + Sync> = match section_ty {
            KmpSectionType::StartingPositions => {
                Box::new(StartingPositions::deserialize(reader, section_header)?)
            }
            KmpSectionType::EnemyRoutePoints => {
                Box::new(EnemyRoutePoints::deserialize(reader, section_header)?)
            }
            KmpSectionType::EnemyRoute => {
                Box::new(EnemyRoutes::deserialize(reader, section_header)?)
            }
            KmpSectionType::ItemRoutePoints => {
                Box::new(ItemRoutePoints::deserialize(reader, section_header)?)
            }
            KmpSectionType::Checkpoints => {
                Box::new(Checkpoints::deserialize(reader, section_header)?)
            }
            KmpSectionType::CheckpointGroups => {
                Box::new(CheckpointGroups::deserialize(reader, section_header)?)
            }
            KmpSectionType::GeoObjects => {
                Box::new(GeoObjects::deserialize(reader, section_header)?)
            }
            KmpSectionType::Routes => {
                Box::new(PointInformationPaths::deserialize(reader, section_header)?)
            }
            KmpSectionType::Areas => Box::new(Areas::deserialize(reader, section_header)?),
            _ => {
                tracing::warn!("unimplemented section type: {:?}", section_header.ty);
                continue;
            }
        };

        let child_key = arena.insert(IrNodeDescriptor {
            label: format!("{:?}", section_ty),
            ty: IrNodeType::KmpSection { ty: section_ty },
            parent: Some(kmp_root),
            contents: ContentSlot::eager(content),
            ..Default::default()
        });

        children.push(child_key);
    }

    arena.insert_at(
        kmp_root,
        IrNodeDescriptor {
            label: name,
            ty: IrNodeType::KmpHeader,
            parent,
            children,
            contents: ContentSlot::eager(Box::new(header))
        }
    );

    Ok(kmp_root)
}
