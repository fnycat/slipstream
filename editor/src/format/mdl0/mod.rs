pub mod bone;
pub mod definitions;
pub mod normals;
pub mod util;
pub mod vertices;

use std::{collections::HashMap, rc::Rc};

use crate::error::{CorruptionError, EditorError, EditorResult, UnsupportedError};
use crate::format::mdl0::bone::deserialize_skeleton;
use crate::r#virtual::defer::Deferred;
use crate::r#virtual::node::{Inspectable, VirtualNode, VirtualNodeBody, VirtualNodeKind};
use crate::r#virtual::refs::{VirtualNodeId, VirtualRefCache, VirtualRefCacheExt};
use crate::{
    format::{
        brres::{self, IndexGroup, Subfile, SubfileHeader, SubfileType},
        encoding::{Deserialize, ReadArrayExt, ReadStringExt},
    },
    shared::util::RefCursor,
};
use byteorder::{BigEndian, ReadBytesExt};

pub const MDL0_MAGIC: [u8; 4] = [0x4d, 0x44, 0x4c, 0x30]; // "MDL0"

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScalingMode {
    Standard,
    Softimage,
    Maya,
}

impl TryFrom<u32> for ScalingMode {
    type Error = EditorError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Standard,
            1 => Self::Softimage,
            2 => Self::Maya,
            v => {
                return Err(CorruptionError {
                    reason: format!("invalid scaling mode: {v} (except 0, 1, 2)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

impl Deserialize for ScalingMode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TextureMatrixMode {
    Maya,
    Xsi,
    ThreeDsMax,
}

impl TryFrom<u32> for TextureMatrixMode {
    type Error = EditorError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Maya,
            1 => Self::Xsi,
            2 => Self::ThreeDsMax,
            v => {
                return Err(CorruptionError {
                    reason: format!("invalid texture matrix mode: {v} (expected 0, 1, 2)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

impl Deserialize for TextureMatrixMode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

pub const MDL0_SECTION_NAMES: &[&str] = &[
    "Definitions",
    "Bones",
    "Vertices",
    "Normals",
    "Colors",
    "UVs",
    "Fur vectors",
    "Fur layers",
    "Materials",
    "TEVs",
    "Objects",
    "TextureLinks",
    "PaletteLinks",
    "UserData",
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SectionType {
    Definitions,
    Bones,
    Vertices,
    Normals,
    Colors,
    UvCoordinates,
    FurVectors,
    FurLayers,
    Materials,
    Tevs,
    Objects,
    TextureLinks,
    PaletteLinks,
    UserData,
}

impl TryFrom<u32> for SectionType {
    type Error = EditorError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Definitions,
            1 => Self::Bones,
            2 => Self::Vertices,
            3 => Self::Normals,
            4 => Self::Colors,
            5 => Self::UvCoordinates,
            6 => Self::FurVectors,
            7 => Self::FurLayers,
            8 => Self::Materials,
            9 => Self::Tevs,
            10 => Self::Objects,
            11 => Self::TextureLinks,
            12 => Self::PaletteLinks,
            13 => Self::UserData,
            v => {
                return Err(CorruptionError {
                    reason: format!("invalid MDL0 section ID: {v} (expected 0-13)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

pub trait SectionDeserialize: Sized {
    fn deserialize_section(reader: &mut RefCursor<[u8]>, header_start: u32) -> EditorResult<Self>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mdl0Header {
    pub file_header_offset: i32,
    pub scaling_mode: ScalingMode,
    pub texture_matrix_mode: TextureMatrixMode,
    pub vertex_count: i32,
    pub face_count: i32,
    pub matrix_count: u32,
    pub require_normalized_matrix_array: bool,
    pub require_texture_matrix_array: bool,
    pub enable_bounding_volume_data: bool,
    pub matrix_table_offset: i32,
    pub bounding_volume_minimum: [f32; 3],
    pub bounding_volume_maximum: [f32; 3],
}

impl Deserialize for Mdl0Header {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let start = reader.position();

        let header_length = reader.read_u32::<BigEndian>()?;
        let file_header_offset = reader.read_i32::<BigEndian>()?;
        let scaling_mode = ScalingMode::deserialize(reader)?;
        let texture_matrix_mode = TextureMatrixMode::deserialize(reader)?;
        let vertex_count = reader.read_i32::<BigEndian>()?;
        let face_count = reader.read_i32::<BigEndian>()?;
        let _unused1 = reader.read_i32::<BigEndian>()?;
        let matrix_count = reader.read_u32::<BigEndian>()?;
        let require_normalized_matrix_array = reader.read_u8()? != 0;
        let require_texture_matrix_array = reader.read_u8()? != 0;
        let enable_bounding_volume_data = reader.read_u8()? != 0;
        let _unknown1 = reader.read_u8()?;
        let matrix_table_offset = reader.read_i32::<BigEndian>()?;
        let bounding_volume_minimum = reader.read_f32_array::<3, BigEndian>()?;
        let bounding_volume_maximum = reader.read_f32_array::<3, BigEndian>()?;

        reader.set_position(start + header_length as u64);

        Ok(Self {
            file_header_offset,
            scaling_mode,
            texture_matrix_mode,
            vertex_count,
            face_count,
            matrix_count,
            require_normalized_matrix_array,
            require_texture_matrix_array,
            enable_bounding_volume_data,
            matrix_table_offset,
            bounding_volume_minimum,
            bounding_volume_maximum,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoneLinkTable {
    /// Maps a matrix index to the singular bone index driving it.
    pub driven_matrices: HashMap<u32, u32>,
    /// Indices of matrices that have multiple bones affecting them.
    pub blended_matrices: Vec<u32>,
    /// Indices of bones that do not deform any geometry.
    pub unconnected_bones: Vec<u32>,
}

impl BoneLinkTable {
    pub fn len(&self) -> usize {
        self.driven_matrices.len() + self.blended_matrices.len() + self.unconnected_bones.len()
    }
}

impl Deserialize for BoneLinkTable {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let entry_count = reader.read_u32::<BigEndian>()?;

        let mut driven = HashMap::new();
        let mut blended = Vec::new();
        let mut unconnected = Vec::new();

        let mut blended_appeared = false;
        for i in 0..entry_count {
            let word = reader.read_u32::<BigEndian>()?;
            if word == 0xFFFFFFFF {
                // Matrix `i` has multiple influences.
                blended.push(i);
                blended_appeared = true;
            } else {
                if blended_appeared {
                    // Bone `word` is not connected to any geometry
                    unconnected.push(word);
                } else {
                    // Matrix `i` is driven by bone `word`
                    driven.insert(i, word);
                }
            };
        }

        Ok(Self {
            driven_matrices: driven,
            blended_matrices: blended,
            unconnected_bones: unconnected,
        })
    }
}

pub fn deserialize_virtual(
    reader: &mut RefCursor<[u8]>,
    parent_id: VirtualNodeId,
    ref_cache: &VirtualRefCache,
    name: String,
) -> EditorResult<VirtualNodeId> {
    let subfile_header = SubfileHeader::deserialize(reader, SubfileType::Mdl0)?;

    let expected_sections =
        brres::get_section_count(SubfileType::Mdl0, subfile_header.subfile_version)?;
    if subfile_header.offsets.len() != expected_sections {
        todo!("invalid section count");
    }

    let mdl0_header = Mdl0Header::deserialize(reader)?;

    let bone_links = BoneLinkTable::deserialize(reader)?;
    let index_group = IndexGroup::deserialize(reader)?;

    let mdl_node_id = ref_cache.next_id();

    let mut files = Vec::with_capacity(subfile_header.offsets.len());
    for (i, &section_offset) in subfile_header.offsets.iter().enumerate() {
        // Loops over sections like `Bones`, `Vertices`, `Normals`...

        if section_offset == 0 {
            // Section does not exist, skip it
            continue;
        }

        let section_ty = SectionType::try_from(i as u32)?;

        let mut reader = reader.clone();
        let ref_cache2 = ref_cache.clone();

        let section_id = ref_cache.next_id();
        let section_parser = move |_data| {
            let section_start = subfile_header.header_start as i64 + section_offset as i64;
            reader.set_position(section_start as u64);

            tracing::debug!("Parsing {section_ty:?}");

            match section_ty {
                SectionType::Bones => deserialize_skeleton(&mut reader, section_id, &ref_cache2),
                _ => Ok(VirtualNodeBody {
                    children: Vec::new(),
                    inspectable: None,
                }),
            }
        };

        let node = VirtualNode::from(VirtualNode {
            label: MDL0_SECTION_NAMES[i].to_owned(),
            id: section_id,
            parent: Some(mdl_node_id),
            kind: VirtualNodeKind::Directory,
            body: Deferred::defer((), section_parser)?,
        });

        ref_cache.insert(section_id, node);
        files.push(section_id);
    }

    let node = VirtualNode::from(VirtualNode {
        label: name,
        id: mdl_node_id,
        parent: Some(parent_id),
        kind: VirtualNodeKind::Directory,
        body: Deferred::evaluated(VirtualNodeBody {
            children: files,
            inspectable: None,
        }),
    });

    ref_cache.insert(mdl_node_id, node);
    Ok(mdl_node_id)
}
