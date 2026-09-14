use std::{collections::HashMap, io::Cursor};

use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    brres::{self, IndexGroup, Subfile, SubfileHeader, SubfileType},
    encoding::{Deserialize, ReadArrayExt, ReadStringExt},
    error::{CorruptionError, EncodingError, EncodingResult},
};

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
    fn deserialize(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
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
    fn deserialize(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SectionIds {
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

impl TryFrom<u32> for SectionIds {
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

trait SectionDeserialize: Sized {
    fn deserialize_section(reader: &mut Cursor<&[u8]>, header_start: u32) -> EncodingResult<Self>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Definitions {}

impl SectionDeserialize for Definitions {
    fn deserialize_section(reader: &mut Cursor<&[u8]>, _header_start: u32) -> EncodingResult<Self> {
        tracing::error!("TODO: draw lists");
        Ok(Self {})
    }
}

const IS_BILLBOARD_CHILD_MASK: u32 = 0x00000400;
const IS_DISPLAY_MATRIX_MASK: u32 = 0x00000200;
const IS_VISIBLE_MASK: u32 = 0x00000100;
const DISABLE_CLASSIC_SCALE_MASK: u32 = 0x00000080;
const APPLY_CHILD_SCALE_COMPENSATE_MASK: u32 = 0x00000040;
const APPLY_SCALE_COMPENSATE_MASK: u32 = 0x00000020;
const SCALE_UNIFORM_MASK: u32 = 0x00000010;
const SCALE_ISOTROPIC_MASK: u32 = 0x00000008;
const ROTATION_ISOTROPIC_MASK: u32 = 0x00000004;
const TRANSLATION_ISOTROPIC_MASK: u32 = 0x00000002;
const USE_IDENTITY_MASK: u32 = 0x00000001;

macro_rules! impl_bone_flags {
    ($($flag:ident),*) => {
        paste::paste! {
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct BoneFlags {
                $(pub $flag: bool),*
            }

            impl Deserialize for BoneFlags {
                fn deserialize(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
                    let word = reader.read_u32::<BigEndian>()?;

                    Ok(Self {
                        $($flag: (word & [< $flag:upper _MASK >]) == [< $flag:upper _MASK >]),*
                    })
                }
            }
        }
    }
}

impl_bone_flags! {
    is_billboard_child, is_display_matrix, is_visible, disable_classic_scale, apply_child_scale_compensate,
    apply_scale_compensate, scale_uniform, scale_isotropic, rotation_isotropic, translation_isotropic, use_identity
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BillboardSetting {
    /// No influence.
    None,
    /// Influenced by rotation of parent node. Z-axis is parallel to camera lens axis.
    InfluencedParallelZ,
    /// Influenced by rotation of parent node. Z-axis points toward camera direction.
    InfluencedDirectZ,
    /// Not influenced by rotation of parent node, restricted by camera's up vector.
    /// Z-axis is parallel to camera lens axis.
    RestrictedParallelZ,
    /// Not influenced by rotation of parent node, restricted by camera's up vector.
    /// Z-axis points toward camera direction.
    RestrictedDirectZ,
    /// Influenced by rotation of parent node and rotates only around Y-axis.
    /// Z-axis is parallel to camera lens axis.
    InfluencedOnlyYParallelZ,
    /// Influenced by rotation of parent node and rotates only around Y-axis.
    /// Z-axis points toward camera direction.
    InfluencedOnlyYDirectZ,
}

impl TryFrom<u32> for BillboardSetting {
    type Error = EncodingError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::None,
            1 => Self::InfluencedParallelZ,
            2 => Self::InfluencedDirectZ,
            3 => Self::RestrictedParallelZ,
            4 => Self::RestrictedDirectZ,
            5 => Self::InfluencedOnlyYParallelZ,
            6 => Self::InfluencedOnlyYDirectZ,
            v => {
                return Err(CorruptionError {
                    reason: format!("invalid bone flag billboard setting: {v} (expected 0-6)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

impl Deserialize for BillboardSetting {
    fn deserialize(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bones {
    pub mdl0_offset: i32,
    pub name_offset: i32,
    pub index: u32,
    pub id: u32,
    pub flags: BoneFlags,
    pub billboard_setting: BillboardSetting,
    pub billboard_transform: u32,
    pub scaling_vector: [f32; 3],
    pub rotation_vector: [f32; 3],
    pub translation_vector: [f32; 3],
    pub bounding_volume_min: [f32; 3],
    pub bounding_volume_max: [f32; 3],
    pub parent_offset: i32,
    pub first_child_offset: i32,
    pub next_sibling_offset: i32,
    pub previous_sibling_offset: i32,
    pub user_data_offset: i32,
    pub transform_matrix: [f32; 12],
    pub inverse_matrix: [f32; 12],
}

impl SectionDeserialize for Bones {
    fn deserialize_section(reader: &mut Cursor<&[u8]>, _header_start: u32) -> EncodingResult<Self> {
        tracing::trace!(
            "Reading model bones section, at location {}",
            reader.position()
        );

        let start = reader.position();

        let length = reader.read_u32::<BigEndian>()?;
        let mdl0_offset = reader.read_i32::<BigEndian>()?;
        let name_offset = reader.read_i32::<BigEndian>()?;
        let index = reader.read_u32::<BigEndian>()?;
        let id = reader.read_u32::<BigEndian>()?;
        let flags = BoneFlags::deserialize(reader)?;
        let billboard_setting = BillboardSetting::deserialize(reader)?;
        let billboard_transform = reader.read_u32::<BigEndian>()?;

        let scaling_vector = reader.read_f32_array::<3, BigEndian>()?;
        let rotation_vector = reader.read_f32_array::<3, BigEndian>()?;
        let translation_vector = reader.read_f32_array::<3, BigEndian>()?;
        let bounding_volume_min = reader.read_f32_array::<3, BigEndian>()?;
        let bounding_volume_max = reader.read_f32_array::<3, BigEndian>()?;
        let parent_offset = reader.read_i32::<BigEndian>()?;
        let first_child_offset = reader.read_i32::<BigEndian>()?;
        let next_sibling_offset = reader.read_i32::<BigEndian>()?;
        let previous_sibling_offset = reader.read_i32::<BigEndian>()?;
        let user_data_offset = reader.read_i32::<BigEndian>()?;
        let transform_matrix = reader.read_f32_array::<12, BigEndian>()?;
        let inverse_matrix = reader.read_f32_array::<12, BigEndian>()?;

        reader.set_position(start + length as u64);

        Ok(Self {
            mdl0_offset,
            name_offset,
            index,
            id,
            flags,
            billboard_setting,
            billboard_transform,
            scaling_vector,
            rotation_vector,
            translation_vector,
            bounding_volume_min,
            bounding_volume_max,
            parent_offset,
            first_child_offset,
            next_sibling_offset,
            previous_sibling_offset,
            user_data_offset,
            transform_matrix,
            inverse_matrix,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VertexData {
    XY(Vec<[f32; 2]>),
    XYZ(Vec<[f32; 3]>),
}

impl VertexData {
    pub fn len(&self) -> usize {
        match self {
            Self::XY(verts) => verts.len(),
            Self::XYZ(verts) => verts.len(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::XY(verts) => bytemuck::cast_slice(verts),
            Self::XYZ(verts) => bytemuck::cast_slice(verts),
        }
    }
}

const COMPONENTS_XY: u32 = 0x0;
const COMPONENTS_XYZ: u32 = 0x1;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentFormat {
    Uint8 = 0,
    Int8 = 1,
    Uint16 = 2,
    Int16 = 3,
    Float = 4,
}

impl TryFrom<u32> for ComponentFormat {
    type Error = EncodingError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Uint8,
            1 => Self::Int8,
            2 => Self::Uint16,
            3 => Self::Int16,
            4 => Self::Float,
            v => {
                return Err(CorruptionError {
                    reason: format!("invalid vertex format: {v} (expected 0-4)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

impl Deserialize for ComponentFormat {
    fn deserialize(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

/// Deserializes vertex or normal components.
fn deserialize_components<const N: usize>(
    reader: &mut Cursor<&[u8]>,
    count: u16,
    format: ComponentFormat,
    divisor: u8,
) -> EncodingResult<Vec<[f32; N]>> {
    let mut components = Vec::with_capacity(count as usize);
    let factor = 1.0 / 2.0f32.powi(divisor as i32);

    match format {
        ComponentFormat::Uint8 => {
            for _ in 0..count {
                let raw_comps = reader.read_u8_array::<N>()?;
                let comps = std::array::from_fn(|i| raw_comps[i] as f32 * factor);

                components.push(comps);
            }
        }
        ComponentFormat::Int8 => {
            for _ in 0..count {
                let raw_comps = reader.read_i8_array::<N>()?;
                let comps = std::array::from_fn(|i| raw_comps[i] as f32 * factor);

                components.push(comps);
            }
        }
        ComponentFormat::Uint16 => {
            for _ in 0..count {
                let raw_comps = reader.read_u16_array::<N, BigEndian>()?;
                let comps = std::array::from_fn(|i| raw_comps[i] as f32 * factor);

                components.push(comps);
            }
        }
        ComponentFormat::Int16 => {
            for _ in 0..count {
                let raw_comps = reader.read_i16_array::<N, BigEndian>()?;
                let comps = std::array::from_fn(|i| raw_comps[i] as f32 * factor);

                components.push(comps);
            }
        }
        ComponentFormat::Float => {
            for _ in 0..count {
                let raw_comps = reader.read_f32_array::<N, BigEndian>()?;

                components.push(raw_comps);
            }
        }
    }

    Ok(components)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vertices {
    /// Offsets are relative to this position.
    pub header_start: u32,
    pub index: u32,
    pub mdl0_offset: i32,
    pub name_offset: i32,
    pub data_offset: i32,
    pub format: ComponentFormat,
    pub divisor: u8,
    pub stride: u8,
    pub bounding_volume_min: [f32; 3],
    pub bounding_volume_max: [f32; 3],
    pub vertices: VertexData,
}

impl SectionDeserialize for Vertices {
    fn deserialize_section(reader: &mut Cursor<&[u8]>, header_start: u32) -> EncodingResult<Self> {
        let _length = reader.read_u32::<BigEndian>()?;
        let mdl0_offset = reader.read_i32::<BigEndian>()?;
        let data_offset = reader.read_i32::<BigEndian>()?;
        let name_offset = reader.read_i32::<BigEndian>()?;
        let index = reader.read_u32::<BigEndian>()?;
        let component_count = reader.read_u32::<BigEndian>()?;
        let format = ComponentFormat::deserialize(reader)?;
        let divisor = reader.read_u8()?;
        let stride = reader.read_u8()?;
        let vertex_count = reader.read_u16::<BigEndian>()?;
        let bounding_volume_min = reader.read_f32_array::<3, BigEndian>()?;
        let bounding_volume_max = reader.read_f32_array::<3, BigEndian>()?;

        tracing::trace!("Reading {vertex_count} vertices");

        let vertices_start = header_start as i64 + data_offset as i64;
        reader.set_position(vertices_start as u64);

        let vertices = match component_count {
            COMPONENTS_XY => VertexData::XY(deserialize_components::<2>(
                reader,
                vertex_count,
                format,
                divisor,
            )?),
            COMPONENTS_XYZ => VertexData::XYZ(deserialize_components::<3>(
                reader,
                vertex_count,
                format,
                divisor,
            )?),
            v => {
                return Err(CorruptionError {
                    reason: format!("invalid vertex component count: {v} (expected 2 or 3)"),
                    location: Some(reader.position()),
                    ..Default::default()
                }
                .into());
            }
        };

        Ok(Self {
            header_start,
            vertices,
            index,
            mdl0_offset,
            name_offset,
            data_offset,
            format,
            divisor,
            stride,
            bounding_volume_min,
            bounding_volume_max,
        })
    }
}

const COMPONENTS_NORMAL: u32 = 0x0;
const COMPONENTS_ALL: u32 = 0x1;
const COMPONENTS_ANY: u32 = 0x2;

#[derive(Debug, Clone, PartialEq)]
pub enum NormalData {
    /// Only the normal.
    Normal(Vec<[f32; 3]>),
    /// Includes all of the normal, bi-normal and tangent
    All(Vec<[f32; 9]>),
    /// Either the normal, bi-normal or tangent.
    Any(Vec<[f32; 3]>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Normals {
    pub header_start: u32,
    pub mdl0_offset: i32,
    pub data_offset: i32,
    pub name_offset: i32,
    pub index: u32,
    pub format: ComponentFormat,
    pub divisor: u8,
    pub stride: u8,
    pub normals: NormalData,
}

impl SectionDeserialize for Normals {
    fn deserialize_section(reader: &mut Cursor<&[u8]>, header_start: u32) -> EncodingResult<Self> {
        let _length = reader.read_u32::<BigEndian>()?;
        let mdl0_offset = reader.read_i32::<BigEndian>()?;
        let data_offset = reader.read_i32::<BigEndian>()?;
        let name_offset = reader.read_i32::<BigEndian>()?;
        let index = reader.read_u32::<BigEndian>()?;
        let component_count = reader.read_u32::<BigEndian>()?;
        let format = ComponentFormat::deserialize(reader)?;
        let divisor = reader.read_u8()?;
        let stride = reader.read_u8()?;
        let normal_count = reader.read_u16::<BigEndian>()?;

        let normals_start = header_start as i64 + data_offset as i64;
        reader.set_position(normals_start as u64);

        let normals = match component_count {
            COMPONENTS_NORMAL => NormalData::Normal(deserialize_components::<3>(
                reader,
                normal_count,
                format,
                divisor,
            )?),
            COMPONENTS_ALL => NormalData::All(deserialize_components::<9>(
                reader,
                normal_count,
                format,
                divisor,
            )?),
            COMPONENTS_ANY => NormalData::Any(deserialize_components::<3>(
                reader,
                normal_count,
                format,
                divisor,
            )?),
            v => {
                return Err(CorruptionError {
                    reason: format!("invalid component count: {v} (expected 0-2)"),
                    location: Some(reader.position()),
                    ..Default::default()
                }
                .into());
            }
        };

        Ok(Self {
            header_start,
            mdl0_offset,
            data_offset,
            name_offset,
            index,
            format,
            divisor,
            stride,
            normals,
        })
    }
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
    fn deserialize(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
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
    fn deserialize(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct Mdl0Subfile {
    pub subfile_header: SubfileHeader,
    pub mdl0_header: Mdl0Header,
    pub bone_links: BoneLinkTable,
    pub definitions: Option<HashMap<String, Definitions>>,
    pub bones: Option<HashMap<String, Bones>>,
    pub vertices: Option<HashMap<String, Vertices>>,
    pub normals: Option<HashMap<String, Normals>>,
}

fn deserialize_section<T: SectionDeserialize>(
    reader: &mut Cursor<&[u8]>,
) -> EncodingResult<HashMap<String, T>> {
    let index_group = IndexGroup::deserialize(reader)?;
    let mut map = HashMap::with_capacity(index_group.entries.len());

    for entry in &index_group.entries[1..] {
        let name = index_group.get_entry_name(reader.get_ref(), entry)?;
        let section_start = index_group.get_entry_data_start(entry);
        reader.set_position(section_start as u64);

        tracing::trace!("Reading entry `{name}`, at location {section_start}");

        map.insert(
            name.to_owned(),
            T::deserialize_section(reader, section_start)?,
        );
    }

    Ok(map)
}

impl Deserialize for Mdl0Subfile {
    fn deserialize(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        tracing::trace!("Reading MDL0 file, at section {}", reader.position());

        let subfile_header = SubfileHeader::deserialize(reader, SubfileType::Mdl0)?;
        let expected_sections =
            brres::get_section_count(SubfileType::Mdl0, subfile_header.subfile_version)?;

        if subfile_header.offsets.len() != expected_sections {
            return Err(CorruptionError {
                reason: format!(
                    "invalid MDL0 section count: expected {expected_sections}, got {}",
                    subfile_header.offsets.len()
                ),
                ..Default::default()
            }
            .into());
        }

        let mdl0_header = Mdl0Header::deserialize(reader)?;

        let bone_links = BoneLinkTable::deserialize(reader)?;
        tracing::debug!("Loaded {} bone links", bone_links.len());

        let index_group = IndexGroup::deserialize(reader)?;
        tracing::debug!("{index_group:?}");

        let mut definitions = None;
        let mut bones = None;
        let mut vertices = None;
        let mut normals = None;

        tracing::error!("TAKING ONLY 4 SECTIONS");
        for (i, &section_offset) in subfile_header.offsets.iter().enumerate().take(4) {
            let section_ty = SectionIds::try_from(i as u32)?;
            if section_offset == 0 {
                tracing::debug!("Section `{section_ty:?}` does not exist, skipping");
                continue;
            }

            let section_start = subfile_header.header_start as i64 + section_offset as i64;
            reader.set_position(section_start as u64);

            tracing::trace!("Reading section `{section_ty:?}`");

            match section_ty {
                SectionIds::Definitions => {
                    definitions = Some(deserialize_section::<Definitions>(reader)?)
                }
                SectionIds::Bones => bones = Some(deserialize_section::<Bones>(reader)?),
                SectionIds::Vertices => vertices = Some(deserialize_section::<Vertices>(reader)?),
                SectionIds::Normals => normals = Some(deserialize_section::<Normals>(reader)?),
                v => todo!("{v:?}"),
            }
        }

        Ok(Self {
            subfile_header,
            mdl0_header,
            bone_links,
            definitions,
            bones,
            vertices,
            normals,
        })
    }
}

impl Subfile for Mdl0Subfile {
    const MAGIC: [u8; 4] = [0x4d, 0x44, 0x4c, 0x30]; // "MDL0"
}
