use byteorder::{BigEndian, ReadBytesExt};
use std::cell::RefMut;
use std::collections::HashMap;
use std::{io::Cursor, rc::Rc};

use crate::error::{CorruptionError, EditorError, EditorResult, InvalidInputError};
use crate::format::brres::IndexGroup;
use crate::r#virtual::defer::Deferred;
use crate::r#virtual::node::{VirtualNode, VirtualNodeBody, VirtualNodeKind};
use crate::r#virtual::refs::{VirtualNodeId, VirtualRefCache, VirtualRefCacheExt};
use crate::{
    format::{
        encoding::{Deserialize, ReadArrayExt},
        mdl0::SectionDeserialize,
    },
    shared::util::RefCursor,
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
                fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
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
    type Error = EditorError;

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
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bone {
    pub bone_start: u32,
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

impl Bone {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
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
            bone_start: start as u32,
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

#[derive(Debug)]
pub struct NamedBone {
    pub name: String,
    pub bone: Bone,
}

/// Similar to [`Bone`] but contains virtual node IDs to reference other bones, instead
/// of using offsets.
pub struct VirtualBone {
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
    pub parent: VirtualNodeId,
    pub user_data_offset: i32,
    pub transform_matrix: [f32; 12],
    pub inverse_matrix: [f32; 12],
}

fn build_skeleton_tree(
    reader: &mut RefCursor<[u8]>,
    bones: &[NamedBone],
    parent_id: VirtualNodeId,
    ref_cache: &VirtualRefCache,
) -> EditorResult<VirtualNodeId> {
    /// The offset between the start of the bone and the bone's index.
    const BONE_INDEX_OFFSET: u64 = 3 * 4;

    // Create a virtual node for each of the bones.
    //
    // These will later be attached to each other to form a skeleton.
    let virtual_bones = bones
        .iter()
        .map(|bone| {
            let id = ref_cache.next_id();
            let node = VirtualNode {
                label: bone.name.clone(),
                id,
                kind: VirtualNodeKind::Container,
                parent: None,
                body: Deferred::evaluated(VirtualNodeBody {
                    children: Vec::new(),
                    inspectable: None,
                }),
            };

            ref_cache.insert(id, node);
            id
        })
        .collect::<Vec<_>>();

    let mut found_root = None; // The bone that was determined to be the root of the skeleton.
    for (i, bone) in bones.iter().enumerate() {
        let curr_id = virtual_bones[i];

        if bone.bone.parent_offset == 0 {
            found_root = Some(i);
            continue; // No parent
        }

        let parent_start = bone.bone.bone_start as i64 + bone.bone.parent_offset as i64;
        reader.set_position(parent_start as u64 + BONE_INDEX_OFFSET);

        // Index into `virtual_bones` of the parent.
        let parent_index = reader.read_u32::<BigEndian>()?;
        if parent_index == i as u32 {
            // Ensure a bone is not its own parent.
            // This would cause cyclical references.

            return Err(InvalidInputError {
                reason: format!("bone `{}` is its own parent", bone.name),
                ..Default::default()
            }
            .into());
        }

        let parent_id = virtual_bones[parent_index as usize];
        let parent_node = ref_cache
            .get(parent_id)
            .ok_or_else(|| {
                EditorError::from(InvalidInputError {
                    reason: format!("virtual node {parent_id} does not exist"),
                    ..Default::default()
                })
        })?;

        parent_node.borrow_mut().body.inspect_mut(|body| {
            body.children.push(curr_id);
        });

        let curr_node = ref_cache
            .get(curr_id)
            .ok_or_else(|| {
                EditorError::from(InvalidInputError {
                    reason: format!("virtual node {curr_id} does not exist"),
                    ..Default::default()
                })
            })?;

        curr_node.borrow_mut().parent = Some(parent_id);
    }

    let Some(root_index) = found_root else {
        return Err(InvalidInputError {
            reason: String::from("skeleton contained no root bone"),
            ..Default::default()
        }.into());
    };
    
    tracing::trace!("Skeleton constructed");

    let root = virtual_bones[root_index];
    Ok(root)
}

pub fn deserialize_skeleton(
    reader: &mut RefCursor<[u8]>,
    parent_id: VirtualNodeId,
    ref_cache: &VirtualRefCache,
) -> EditorResult<VirtualNodeBody> {
    let section_index = IndexGroup::deserialize(reader)?;
    let mut bones = Vec::with_capacity(section_index.entries.len() - 1);

    for entry in &section_index.entries[1..] {
        let name = section_index.get_entry_name(reader, entry)?;

        let data_start = section_index.get_entry_data_start(entry);
        reader.set_position(data_start as u64);

        let bone = Bone::deserialize(reader)?;
        bones.push(NamedBone { name, bone });
    }

    let node = build_skeleton_tree(reader, &bones, parent_id, ref_cache)?;

    Ok(VirtualNodeBody {
        inspectable: None,
        children: vec![node],
    })
}
