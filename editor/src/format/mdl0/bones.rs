use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt};

use crate::format::{
    encoding::{Deserialize, ReadArrayExt},
    error::{CorruptionError, EncodingError, EncodingResult},
    mdl0::mdl0::SectionDeserialize,
};

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

impl BillboardSetting {
    pub fn len() -> usize {
        Self::InfluencedOnlyYDirectZ as usize + 1
    }
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
