mod bones;
mod colors;
mod definitions;
mod fur_layers;
mod fur_vectors;
mod materials;
mod normals;
mod pal_links;
mod polygon;
mod section;
mod tevs;
mod tex_links;
mod user_data;
mod uvs;
mod vertices;
mod visitor;

pub use bones::*;
pub use colors::*;
pub use definitions::*;
pub use fur_layers::*;
pub use fur_vectors::*;
pub use materials::*;
pub use normals::*;
pub use pal_links::*;
pub use polygon::*;
pub use section::*;
pub use tevs::*;
pub use tex_links::*;
pub use user_data::*;
pub use uvs::*;
pub use vertices::*;
pub use visitor::*;

use std::ops::ControlFlow;

use crate::SAFE_ALLOCATION_CAPACITY;
use crate::brres::{self, SubfileHeader, SubfileType};
use crate::encoding::{Deserialize, ReadArrayExt};
use crate::node::arena::{IrArena, IrNodeDescriptor, IrNodeKey};
use crate::node::node::{ContentSlot, IrNode, IrNodeType};
use crate::util::Box3;
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};
use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::{Inspect, try_from};
use slipstream_shared::cursor::RefCursor;
use slipstream_shared::error::{
    CorruptionError, SlipstreamError, SlipstreamResult, UnsupportedError
};
use slipstream_shared::verify;

pub const MDL_MAGIC: [u8; 4] = [0x4d, 0x44, 0x4c, 0x30]; // "MDL"

#[try_from(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Inspect)]
pub enum ScalingMode {
    Standard = 0x00,
    #[inspect(rename = "Autodesk Softimage")]
    Softimage = 0x01,
    #[inspect(rename = "Autodesk Maya")]
    Maya = 0x02
}

impl ScalingMode {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

#[try_from(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Inspect)]
pub enum TextureMatrixMode {
    #[inspect(rename = "Autodesk Maya")]
    Maya = 0x00,
    #[inspect(rename = "Autodesk Softimage")]
    Softimage = 0x01,
    #[inspect(rename = "3DS Max")]
    ThreeDsMax = 0x02
}

impl TextureMatrixMode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

pub const MDL_SECTION_NAMES: &[&str] = &[
    "Bytecode",
    "Bones",
    "Vertices",
    "Normals",
    "Colors",
    "UVs",
    "Fur vectors",
    "Fur layers",
    "Materials",
    "TEVs",
    "Shapes",
    "Texture links",
    "Palette links",
    "User data"
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SectionType {
    DrawLists,
    Bones,
    Vertices,
    Normals,
    Colors,
    UvCoordinates,
    FurVectors,
    FurLayers,
    Materials,
    Tevs,
    Shapes,
    TextureLinks,
    PaletteLinks,
    UserData
}

impl TryFrom<u32> for SectionType {
    type Error = SlipstreamError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::DrawLists,
            1 => Self::Bones,
            2 => Self::Vertices,
            3 => Self::Normals,
            4 => Self::Colors,
            5 => Self::UvCoordinates,
            6 => Self::FurVectors,
            7 => Self::FurLayers,
            8 => Self::Materials,
            9 => Self::Tevs,
            10 => Self::Shapes,
            11 => Self::TextureLinks,
            12 => Self::PaletteLinks,
            13 => Self::UserData,
            v => {
                return Err(CorruptionError {
                    reason: format!("invalid MDL section ID: {v} (expected 0-13)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[repr(u8)]
pub enum EnvelopeMatrixMode {
    Normal = 0,
    Approximate = 1,
    Exact = 2
}

impl TryFrom<u8> for EnvelopeMatrixMode {
    type Error = SlipstreamError;

    fn try_from(value: u8) -> SlipstreamResult<Self> {
        Ok(match value {
            0 => Self::Normal,
            1 => Self::Approximate,
            2 => Self::Exact,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid envelope matrix mode: {value}, expected 0, 1 or 2"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

impl Deserialize for EnvelopeMatrixMode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let byte = reader.read_u8()?;
        Self::try_from(byte)
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct MatrixTable {
    pub entries: Vec<i32>
}

impl MatrixTable {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let count = reader.read_u32::<BigEndian>()?;

        let count_cap = if count as usize > SAFE_ALLOCATION_CAPACITY {
            tracing::error!(
                "MDL matrix table exceeds maximum safe allocation capacity ({} > {SAFE_ALLOCATION_CAPACITY}), the file is likely corrupted.",
                count
            );
            SAFE_ALLOCATION_CAPACITY as u32
        } else {
            count
        };

        let mut entries = Vec::with_capacity(count_cap as usize);
        for _ in 0..count {
            entries.push(reader.read_i32::<BigEndian>()?);
        }

        Ok(Self { entries })
    }
}

pub trait SectionDeserialize: Sized {
    fn deserialize_section(
        reader: &mut RefCursor<[u8]>,
        header_start: u32
    ) -> SlipstreamResult<Self>;
}

#[derive(Debug, Clone, PartialEq, Inspect)]
#[inspect(rename = "MDL Header")]
pub struct Mdl0Header {
    #[inspect(rename = "MDL Start Offset")]
    #[inspect(tooltip = "Offset in bytes to the start of the MDL file.")]
    pub mdl_offset: i32,
    pub scaling_mode: ScalingMode,
    pub texture_matrix_mode: TextureMatrixMode,
    pub vertex_count: i32,
    pub triangle_count: i32,
    pub name_offset: i32,
    pub node_count: u32,
    pub needs_normal_matrix_array: bool,
    pub needs_texture_matrix_array: bool,
    pub enable_bounding_volumes: bool,
    pub envelope_matrix_mode: EnvelopeMatrixMode,
    pub bounding_volume: Box3,
    #[inspect(opened = false)]
    pub matrix_table: MatrixTable
}

impl Mdl0Header {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let start = reader.position();

        let header_length = reader.read_u32::<BigEndian>()?;
        let mdl_offset = reader.read_i32::<BigEndian>()?;
        let scaling_mode = ScalingMode::deserialize(reader)?;
        let texture_matrix_mode = TextureMatrixMode::deserialize(reader)?;
        let vertex_count = reader.read_i32::<BigEndian>()?;
        let triangle_count = reader.read_i32::<BigEndian>()?;
        let name_offset = reader.read_i32::<BigEndian>()?;
        let node_count = reader.read_u32::<BigEndian>()?;
        let needs_normal_matrix_array = reader.read_u8()? != 0;
        let needs_texture_matrix_array = reader.read_u8()? != 0;
        let enable_bounding_volumes = reader.read_u8()? != 0;
        let envelope_matrix_mode = EnvelopeMatrixMode::deserialize(reader)?;
        let data_offset = reader.read_u32::<BigEndian>()?;

        let bounding_volume = Box3::deserialize(reader)?;

        tracing::warn!("offset: {:#04x}", reader.position() - start);

        reader.set_position(start + data_offset as u64);

        let matrix_table = MatrixTable::deserialize(reader)?;

        tracing::trace!(
            "Read {}, must have {}",
            reader.position(),
            start + header_length as u64
        );
        reader.set_position(start + header_length as u64);

        Ok(Self {
            mdl_offset,
            scaling_mode,
            texture_matrix_mode,
            vertex_count,
            triangle_count,
            name_offset,
            node_count,
            needs_normal_matrix_array,
            needs_texture_matrix_array,
            enable_bounding_volumes,
            envelope_matrix_mode,
            bounding_volume,
            matrix_table
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct BoneLinkTable {
    /// Maps a matrix index to the singular bone index driving it.
    pub rigid_bones: Vec<u32>,
    /// Indices of matrices that have multiple bones affecting them.
    pub mixed_bones: Vec<u32>,
    /// Indices of bones that do not deform any geometry.
    pub unconnected_bones: Vec<u32>
}

impl BoneLinkTable {
    /// Returns the amount of entries in this table.
    pub fn len(&self) -> usize {
        self.rigid_bones.len() + self.mixed_bones.len() + self.unconnected_bones.len()
    }
}

impl BoneLinkTable {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let entry_count = reader.read_u32::<BigEndian>()?;

        let mut rigid = Vec::new();
        let mut mixed = Vec::new();
        let mut unconnected = Vec::new();

        let mut is_mixed = false;
        for i in 0..entry_count {
            let word = reader.read_u32::<BigEndian>()?;
            if word == 0xFFFFFFFF {
                mixed.push(i);
                is_mixed = true;
            } else {
                if is_mixed {
                    unconnected.push(i);
                } else {
                    rigid.push(i);
                }
            }
        }

        if !mixed.is_empty() || !unconnected.is_empty() {
            // Seems like the mixed entries are extra matrices for the transform storage buffer
            // on the GPU. The Boo model from 17.szs references bone 9, which is not an existing bone.
            // It is however in both the mixed bone link table list and the nodemix weights.
            tracing::warn!(
                "bone link table has mixed or unconnected entries, these are currently not properly supported"
            );
        }

        Ok(Self {
            rigid_bones: rigid,
            mixed_bones: mixed,
            unconnected_bones: unconnected
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
#[inspect(rename = "MDL Root")]
pub struct Mdl0Root {
    pub header: Mdl0Header,
    pub bone_link_table: BoneLinkTable
}

impl Visitable for Mdl0Root {
    fn accept(&self, node: VisitorCtxMeta<'_>, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_mdl()
            .map(|vis| vis.visit_mdl(VisitorCtx::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut<'_>,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_mdl()
            .map(|vis| vis.visit_mdl_mut(VisitorCtxMut::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}

#[tracing::instrument(skip_all, fields(name, parent_id))]
pub fn deserialize(
    reader: &mut RefCursor<[u8]>,
    parent_id: Option<IrNodeKey>,
    arena: &IrArena,
    name: String
) -> SlipstreamResult<IrNodeKey> {
    tracing::trace!("Opening {name}");

    let subfile_header = SubfileHeader::deserialize(reader, SubfileType::Mdl)?;
    if subfile_header.version != 11 {
        return Err(UnsupportedError {
            reason: format!(
                "MDL version {} is not supported, only version 11 is",
                subfile_header.version
            ),
            location: Some(reader.position())
        }
        .into());
    }

    let expected_sections = brres::get_section_count(SubfileType::Mdl, subfile_header.version)?;
    verify!(
        subfile_header.offsets.len() == expected_sections,
        "MDL section count ({}) did not match expected section count ({expected_sections})",
        subfile_header.offsets.len()
    );

    let mdl_header = Mdl0Header::deserialize(reader)?;

    let bone_link_table = BoneLinkTable::deserialize(reader)?;
    let mdl_root_key = arena.reserve_key();

    let file_count = subfile_header.offsets.iter().filter(|&o| *o != 0).count();
    let mut files = Vec::with_capacity(file_count);
    for (i, &section_offset) in subfile_header.offsets.iter().enumerate() {
        // Loops over sections like `Bones`, `Vertices`, `Normals`...

        if section_offset == 0 {
            // Section does not exist, skip it
            continue;
        }

        let section_ty = SectionType::try_from(i as u32)?;

        let mut reader = reader.clone();

        let section_start = subfile_header.header_start as i64 + section_offset as i64;
        reader.set_position(section_start as u64);

        let section_key = match section_ty {
            SectionType::DrawLists => deserialize_leaf_section::<Definitions>(
                &mut reader,
                subfile_header.header_start,
                Some(mdl_root_key),
                arena
            ),
            SectionType::Bones => {
                bones::deserialize_skeleton(&mut reader, Some(mdl_root_key), arena)
            }
            SectionType::Vertices => deserialize_leaf_section::<VertexBuffer>(
                &mut reader,
                subfile_header.header_start,
                Some(mdl_root_key),
                arena
            ),
            SectionType::Normals => deserialize_leaf_section::<NormalBuffer>(
                &mut reader,
                subfile_header.header_start,
                Some(mdl_root_key),
                arena
            ),
            SectionType::Colors => deserialize_leaf_section::<ColorBuffer>(
                &mut reader,
                subfile_header.header_start,
                Some(mdl_root_key),
                arena
            ),
            SectionType::UvCoordinates => deserialize_leaf_section::<UvBuffer>(
                &mut reader,
                subfile_header.header_start,
                Some(mdl_root_key),
                arena
            ),
            SectionType::Materials => deserialize_leaf_section::<MaterialBuffer>(
                &mut reader,
                subfile_header.header_start,
                Some(mdl_root_key),
                arena
            ),
            SectionType::Tevs => deserialize_leaf_section::<Tev>(
                &mut reader,
                subfile_header.header_start,
                Some(mdl_root_key),
                arena
            ),
            SectionType::Shapes => deserialize_leaf_section::<Polygon>(
                &mut reader,
                subfile_header.header_start,
                Some(mdl_root_key),
                arena
            ),
            SectionType::TextureLinks => deserialize_leaf_section::<TextureLinks>(
                &mut reader,
                subfile_header.header_start,
                Some(mdl_root_key),
                arena
            ),
            SectionType::PaletteLinks => deserialize_leaf_section::<PaletteLinks>(
                &mut reader,
                subfile_header.header_start,
                Some(mdl_root_key),
                arena
            ),
            SectionType::FurLayers => deserialize_leaf_section::<FurLayers>(
                &mut reader,
                subfile_header.header_start,
                Some(mdl_root_key),
                arena
            ),
            SectionType::FurVectors => deserialize_leaf_section::<FurVectors>(
                &mut reader,
                subfile_header.header_start,
                Some(mdl_root_key),
                arena
            ),
            SectionType::UserData => deserialize_leaf_section::<UserData>(
                &mut reader,
                subfile_header.header_start,
                Some(mdl_root_key),
                arena
            )
        }?;

        files.push(section_key);
    }

    arena.insert_at(
        mdl_root_key,
        IrNodeDescriptor {
            label: name,
            ty: IrNodeType::Mdl0Root,
            parent: parent_id,
            children: files,
            contents: ContentSlot::eager(Box::new(Mdl0Root {
                header: mdl_header,
                bone_link_table
            }))
        }
    );

    Ok(mdl_root_key)
}

/// General section header that fits most MDL sections.
///
/// Some exceptions to this are `Bones` (no data offset).
#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct SectionHeader {
    /// Cursor position pointing to the start of this section file.
    #[inspect(tooltip = "Cursor position where this header starts in the buffer.")]
    pub section_start: u64,
    /// Length of this section file in bytes.
    #[inspect(tooltip = "Length of this section in bytes.")]
    pub length: u32,
    /// Offset to the start of the MDL file. This is relative to `section_start`.
    /// The offset is generally negative.
    #[inspect(rename = "MDL Start Offset")]
    #[inspect(tooltip = "Offset back to the start of the MDL file.")]
    pub mdl_offset: i32,
    /// Offset to the data of this section file. Not all sections use this offset, but
    /// for example the [`VertexBuffer`] section stores its raw vertex data at this offset.
    #[inspect(tooltip = "Offset to the start of the data section of this file.")]
    pub data_offset: i32,
    /// Offset to the name of this section file.
    #[inspect(tooltip = "Offset to the name of this file.")]
    pub name_offset: i32,
    /// Index of this section file. This is generally just based on the location within the parent folder.
    #[inspect(tooltip = "Index within the MDL subfolder.")]
    pub index: u32
}

impl SectionHeader {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let section_start = reader.position();
        let length = reader.read_u32::<BigEndian>()?;
        let mdl_offset = reader.read_i32::<BigEndian>()?;
        let data_offset = reader.read_i32::<BigEndian>()?;
        let name_offset = reader.read_i32::<BigEndian>()?;
        let index = reader.read_u32::<BigEndian>()?;

        Ok(Self {
            section_start,
            length,
            mdl_offset,
            data_offset,
            name_offset,
            index
        })
    }

    /// Returns the cursor position that points to the start of this section file's data.
    #[inline]
    pub fn get_data_start(&self) -> u64 {
        (self.section_start as i64 + self.data_offset as i64) as u64
    }
}
