mod ktpt;
mod visitor;

use std::ops::ControlFlow;

pub use ktpt::*;
pub use visitor::*;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::Inspect;
use slipstream_shared::{RefCursor, SlipstreamResult, verify};

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

pub const KMP_READABLE_SECTION_NAMES: [&str; 15] = [
    "Starting Positions",
    "Enemy Route Points",
    "Enemy Route Point Groups",
    "Item Route Points",
    "Item Route Point Groups",
    "Checkpoints",
    "Checkpoint Groups",
    "Geo Objects",
    "Routes",
    "Areas",
    "Cameras",
    "Respawn Positions",
    "Cannon Target Positions",
    "End Battle Positions",
    "Stage Information"
];

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct KmpHeader {
    #[inspect(tooltip = "The length in bytes of the entire KMP file.")]
    pub length: u32,
    #[inspect(tooltip = "Offsets to the subsections within the KMP file.")]
    pub offsets: Vec<u32> // #[inspect(rename = "KTPT Offset")]
                          // pub ktpt_offset: u32,
                          // #[inspect(rename = "ENPT Offset")]
                          // pub enpt_offset: u32,
                          // #[inspect(rename = "ENPH Offset")]
                          // pub enph_offset: u32,
                          // #[inspect(rename = "ITPT Offset")]
                          // pub itpt_offset: u32,
                          // #[inspect(rename = "ITPH Offset")]
                          // pub itph_offset: u32,
                          // #[inspect(rename = "CKPT Offset")]
                          // pub ckpt_offset: u32,
                          // #[inspect(rename = "CKPH Offset")]
                          // pub ckph_offset: u32,
                          // #[inspect(rename = "GOBJ Offset")]
                          // pub gobj_offset: u32,
                          // #[inspect(rename = "POTI Offset")]
                          // pub poti_offset: u32,
                          // #[inspect(rename = "AREA Offset")]
                          // pub area_offset: u32,
                          // #[inspect(rename = "CAME Offset")]
                          // pub came_offset: u32,
                          // #[inspect(rename = "JGPT Offset")]
                          // pub jgpt_offset: u32,
                          // #[inspect(rename = "CNPT Offset")]
                          // pub cnpt_offset: u32,
                          // #[inspect(rename = "MSPT Offset")]
                          // pub mspt_offset: u32,
                          // #[inspect(rename = "STGI Offset")]
                          // pub stgi_offset: u32
}

impl KmpHeader {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let magic = reader.read_u8_array::<4>()?;
        verify!(
            magic == KMP_MAGIC,
            "KMP file magic was expected to be `RKMD`, was actually `{}`. This is not a KMP file",
            String::from_utf8_lossy(&magic)
        );

        let length = reader.read_u32::<BigEndian>()?;
        let mut section_number = reader.read_u16::<BigEndian>()?;
        let header_length = reader.read_u16::<BigEndian>()?;

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

        Ok(Self { length, offsets })

        //         let ktpt_offset = reader.read_u32::<BigEndian>()?;
        //         let enpt_offset = reader.read_u32::<BigEndian>()?;
        //         let enph_offset = reader.read_u32::<BigEndian>()?;
        //         let itpt_offset = reader.read_u32::<BigEndian>()?;
        //         let itph_offset = reader.read_u32::<BigEndian>()?;
        //         let ckpt_offset = reader.read_u32::<BigEndian>()?;
        //         let ckph_offset = reader.read_u32::<BigEndian>()?;
        //         let gobj_offset = reader.read_u32::<BigEndian>()?;
        //         let poti_offset = reader.read_u32::<BigEndian>()?;
        //         let area_offset = reader.read_u32::<BigEndian>()?;
        //         let came_offset = reader.read_u32::<BigEndian>()?;
        //         let jgpt_offset = reader.read_u32::<BigEndian>()?;
        //         let cnpt_offset = reader.read_u32::<BigEndian>()?;
        //         let mspt_offset = reader.read_u32::<BigEndian>()?;
        //         let stgi_offset = reader.read_u32::<BigEndian>()?;
        //
        //         Ok(Self {
        //             length,
        //             ktpt_offset,
        //             enpt_offset,
        //             enph_offset,
        //             itpt_offset,
        //             itph_offset,
        //             ckpt_offset,
        //             ckph_offset,
        //             gobj_offset,
        //             poti_offset,
        //             area_offset,
        //             came_offset,
        //             jgpt_offset,
        //             cnpt_offset,
        //             mspt_offset,
        //             stgi_offset
        //         })
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

pub fn deserialize(
    reader: &mut RefCursor<[u8]>,
    parent: Option<IrNodeKey>,
    arena: &IrArena,
    name: String
) -> SlipstreamResult<IrNodeKey> {
    let header = KmpHeader::deserialize(reader)?;

    let kmp_root = arena.reserve_key();

    // let kmp_section_cap = if header.
    // let mut children = Vec::with_capacity(kmp_section_cap);

    arena.insert_at(
        kmp_root,
        IrNodeDescriptor {
            label: name,
            ty: IrNodeType::KmpHeader,
            parent,
            children: Vec::new(),
            contents: ContentSlot::eager(Box::new(header))
        }
    );

    Ok(kmp_root)
}
