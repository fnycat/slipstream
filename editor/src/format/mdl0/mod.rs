pub mod bones;
pub mod definitions;
pub mod normals;
pub mod util;
pub mod vertices;

use std::{collections::HashMap, io::Cursor, rc::Rc};

use byteorder::{BigEndian, ReadBytesExt};
use egui::accesskit::Role::Section;

use crate::{
    format::{
        brres::{self, IndexGroup, Subfile, SubfileHeader, SubfileType},
        encoding::{Deserialize, ReadArrayExt, ReadStringExt},
        error::{CorruptionError, EncodingError, EncodingResult},
        mdl0::{bones::Bones, definitions::Definitions, normals::Normals, vertices::Vertices},
    },
    shared::{
        defer::Deferred,
        refs::VirtualRefCache,
        util::RefCursor,
        r#virtual::{
            Inspectable, VirtualNode, VirtualNodeContent, VirtualNodeKind, VirtualNodeRef,
        },
    },
};

pub const MDL0_MAGIC: [u8; 4] = [0x4d, 0x44, 0x4c, 0x30]; // "MDL0"

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScalingMode {
    Standard,
    Softimage,
    Maya,
}

impl TryFrom<u32> for ScalingMode {
    type Error = EncodingError;

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
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EncodingResult<Self> {
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
    type Error = EncodingError;

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
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EncodingResult<Self> {
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
    type Error = EncodingError;

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
    fn deserialize_section(reader: &mut RefCursor<[u8]>, header_start: u32)
    -> EncodingResult<Self>;
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
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EncodingResult<Self> {
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
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EncodingResult<Self> {
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
    ref_cache: &mut VirtualRefCache,
    name: String,
) -> EncodingResult<VirtualNodeRef> {
    let subfile_header = SubfileHeader::deserialize(reader, SubfileType::Mdl0)?;

    let expected_sections =
        brres::get_section_count(SubfileType::Mdl0, subfile_header.subfile_version)?;
    if subfile_header.offsets.len() != expected_sections {
        todo!("invalid section count");
    }

    let mdl0_header = Mdl0Header::deserialize(reader)?;

    let bone_links = BoneLinkTable::deserialize(reader)?;
    let index_group = IndexGroup::deserialize(reader)?;

    let mut files = Vec::with_capacity(subfile_header.offsets.len());
    for (i, &section_offset) in subfile_header.offsets.iter().enumerate() {
        // Loops over sections like `Bones`, `Vertices`, `Normals`...

        if section_offset == 0 {
            // Section does not exist, skip it
            continue;
        }

        let section_start = subfile_header.header_start as i64 + section_offset as i64;
        reader.set_position(section_start as u64);

        let section_index = IndexGroup::deserialize(reader)?;
        let mut children = Vec::with_capacity(section_index.entries.len() - 1);

        for (j, entry) in section_index.entries[1..].iter().enumerate() {
            let name = section_index.get_entry_name(reader, entry)?.to_owned();

            let data_start = section_index.get_entry_data_start(entry);

            // Add 2 because we start the iterator on the first element.
            let data_end = if let Some(next) = section_index.entries.get(j + 2) {
                // Take data until next index entry.
                section_index.get_entry_data_start(next) as usize
            } else {
                // Take data until end of MDL0 file.
                subfile_header.header_start as usize + subfile_header.subfile_length as usize
            };

            tracing::debug!("Data end of {name} is {data_end}");

            // let data = reader.get_ref()[data_start as usize..data_end].to_vec();
            reader.set_position(data_start as u64);

            // Cloning is cheap due to the reference counter.
            //
            // The reader must be cloned since it is reused by the next iteration of the loop.
            let mut reader = reader.clone();
            let parse_fn = move |_payload| -> eyre::Result<VirtualNodeContent> {
                let section_ty = SectionType::try_from(i as u32)?;
                tracing::trace!("Lazily evaluating section of type  `{section_ty:?}`");

                let inspectable: Box<dyn Inspectable> = match section_ty {
                    SectionType::Definitions => {
                        Box::new(Definitions::deserialize_section(&mut reader, data_start)?)
                    }
                    SectionType::Bones => {
                        Box::new(Bones::deserialize_section(&mut reader, data_start)?)
                    }
                    SectionType::Vertices => {
                        Box::new(Vertices::deserialize_section(&mut reader, data_start)?)
                    }
                    SectionType::Normals => {
                        Box::new(Normals::deserialize_section(&mut reader, data_start)?)
                    }
                    _ => eyre::bail!(
                        "Evaluation of section of type `{section_ty:?}` is not implemented yet"
                    ),
                };

                Ok(VirtualNodeContent {
                    inspectable: Some(inspectable),
                    children: Vec::new(),
                })
            };

            let id = ref_cache.next_id();
            let node = VirtualNodeRef::from(VirtualNode {
                label: name,
                id,
                kind: VirtualNodeKind::Terminal,
                content: Deferred::defer((), parse_fn),
            });

            ref_cache.insert(id, Rc::downgrade(&node));
            children.push(node);
        }

        let id = ref_cache.next_id();
        let node = VirtualNodeRef::from(VirtualNode {
            label: MDL0_SECTION_NAMES[i].to_owned(),
            id,
            kind: VirtualNodeKind::Container,
            content: Deferred::evaluated(VirtualNodeContent {
                inspectable: None,
                children,
            }),
        });

        ref_cache.insert(id, Rc::downgrade(&node));
        files.push(node);
    }

    let id = ref_cache.next_id();
    let node = VirtualNodeRef::from(VirtualNode {
        label: name,
        id,
        kind: VirtualNodeKind::Container,
        content: Deferred::evaluated(VirtualNodeContent {
            inspectable: None,
            children: files,
        }),
    });

    ref_cache.insert(id, Rc::downgrade(&node));
    Ok(node)
}
