use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use slipstream_derive::{Inspect, inspect_bitfield};
use slipstream_shared::cursor::{MutCursor, RefCursor};
use slipstream_shared::error::{
    CorruptionError, InvalidInputError, SlipstreamError, SlipstreamResult
};
use slipstream_shared::try_unwrap;

use crate::brres::MdlSectionType;
use crate::encoding::ReadArrayExt;
use crate::index::IndexGroup;
use crate::node::arena::{IrArena, IrNodeDescriptor, IrNodeKey};
use crate::node::node::{ContentSlot, IrNodeType};
use crate::util::Box3;
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(summary = "BoneFlags::summary")]
pub struct BoneFlags {
    pub use_identity: bool,
    pub translation_isotropic: bool,
    pub rotation_isotropic: bool,
    pub scale_isotropic: bool,
    pub scale_uniform: bool,
    pub apply_scale_compensate: bool,
    pub apply_child_scale_compensate: bool,
    pub disable_classic_scale: bool,
    pub is_visible: bool,
    pub is_display_matrix: bool,
    pub is_billboard_child: bool,
    #[bits(21)]
    _unused: u32
}

impl BoneFlags {
    #[inline]
    pub fn summary(&self) -> String {
        format!("{:#08x}", self.0)
    }
}

/// Configures the way billboarding is used for this object.
///
/// This can be used to make something always face the camera.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Inspect)]
pub enum BillboardSetting {
    /// No influence.
    #[inspect(tooltip = "The bone does not use any billboard functionality.")]
    Disabled,
    /// Influenced by rotation of parent node. Z-axis is parallel to camera lens axis.
    Billboard,
    /// Influenced by rotation of parent node. Z-axis points toward camera direction.
    PerspectiveBillboard,
    /// Not influenced by rotation of parent node, restricted by camera's up vector.
    /// Z-axis is parallel to camera lens axis.
    CameraBillboard,
    /// Not influenced by rotation of parent node, restricted by camera's up vector.
    /// Z-axis points toward camera direction.
    CameraPerspectiveBillboard,
    /// Influenced by rotation of parent node and rotates only around Y-axis.
    /// Z-axis is parallel to camera lens axis.
    YBillboard,
    /// Influenced by rotation of parent node and rotates only around Y-axis.
    /// Z-axis points toward camera direction.
    YPerspectiveBillboard
}

impl BillboardSetting {
    /// Returns the amount of items in this enum.
    pub fn len() -> usize {
        Self::YPerspectiveBillboard as usize + 1
    }
}

impl TryFrom<u32> for BillboardSetting {
    type Error = SlipstreamError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Disabled,
            1 => Self::Billboard,
            2 => Self::PerspectiveBillboard,
            3 => Self::CameraBillboard,
            4 => Self::CameraPerspectiveBillboard,
            5 => Self::YBillboard,
            6 => Self::YPerspectiveBillboard,
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

impl BillboardSetting {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }

    fn serialize(self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(self as u32)?;
        Ok(())
    }
}

/// A bone that already has all its data deserialized but without resolved references.
///
/// When constructing the skeleton, a new [`Bone`] is created that contains proper references
/// to other bones.
#[derive(Debug, Clone, PartialEq)]
pub struct UnresolvedBone {
    pub bone_start: u32,
    pub index: u32,
    pub id: u32,
    pub flags: BoneFlags,
    /// Configures how billboarding is used for this bone.
    pub billboard_setting: BillboardSetting,
    pub billboard_transform: u32,
    pub scaling_vector: glam::Vec3,
    pub rotation_vector: glam::Vec3,
    pub translation_vector: glam::Vec3,
    pub bounding_volume: Box3,
    /// The offset in bytes to the parent of this bone.
    pub parent_offset: i32,
    pub first_child_offset: i32,
    pub next_sibling_offset: i32,
    pub previous_sibling_offset: i32,
    pub user_data_offset: i32,
    pub transform_matrix: glam::Mat4,
    pub inverse_matrix: glam::Mat4
}

impl UnresolvedBone {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let start = reader.position();

        let _length = reader.read_u32::<BigEndian>()?;
        let _mdl_offset = reader.read_i32::<BigEndian>()?;
        let _name_offset = reader.read_i32::<BigEndian>()?;
        let index = reader.read_u32::<BigEndian>()?;

        let id = reader.read_u32::<BigEndian>()?;
        let flags = BoneFlags::from_bits(reader.read_u32::<BigEndian>()?);
        let billboard_setting = BillboardSetting::deserialize(reader)?;
        let billboard_transform = reader.read_u32::<BigEndian>()?;

        let scaling_vector = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let rotation_vector = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let translation_vector = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let bounding_volume = Box3::deserialize(reader)?;
        let parent_offset = reader.read_i32::<BigEndian>()?;
        let first_child_offset = reader.read_i32::<BigEndian>()?;
        let next_sibling_offset = reader.read_i32::<BigEndian>()?;
        let previous_sibling_offset = reader.read_i32::<BigEndian>()?;
        let user_data_offset = reader.read_i32::<BigEndian>()?;

        let m = reader.read_f32_array::<12, BigEndian>()?;
        let transform_matrix = glam::mat4(
            glam::vec4(m[0], m[4], m[8], 0.0),
            glam::vec4(m[1], m[5], m[9], 0.0),
            glam::vec4(m[2], m[6], m[10], 0.0),
            glam::vec4(m[3], m[7], m[11], 1.0)
        );

        let m = reader.read_f32_array::<12, BigEndian>()?;
        let inverse_matrix = glam::mat4(
            glam::vec4(m[0], m[4], m[8], 0.0),
            glam::vec4(m[1], m[5], m[9], 0.0),
            glam::vec4(m[2], m[6], m[10], 0.0),
            glam::vec4(m[3], m[7], m[11], 1.0)
        );

        Ok(Self {
            bone_start: start as u32,
            index,
            id,
            flags,
            billboard_setting,
            billboard_transform,
            scaling_vector,
            rotation_vector,
            translation_vector,
            bounding_volume,
            parent_offset,
            first_child_offset,
            next_sibling_offset,
            previous_sibling_offset,
            user_data_offset,
            transform_matrix,
            inverse_matrix
        })
    }
}

/// A bone combined with its name.
#[derive(Debug)]
pub struct LabeledBone {
    pub label: String,
    pub data: UnresolvedBone
}

#[derive(Debug, Inspect)]
pub struct Bone {
    /// Regular MDL index of this section. As bones are stored as a linear array of "files" within the MDL file,
    /// these indices correspond to the index of this bone into this array.
    pub index: u32,
    /// The ID stored inside the bone.
    pub id: u32,
    pub flags: BoneFlags,
    #[inspect(opened)]
    pub billboard_setting: BillboardSetting,
    pub billboard_reference: Option<IrNodeKey>,
    pub translation: glam::Vec3,
    #[inspect(suffix = " °")]
    pub rotation: glam::Vec3,
    pub scale: glam::Vec3,
    pub bounding_volume: Box3,
    #[inspect(ignore)]
    pub parent: Option<IrNodeKey>,
    #[inspect(ignore)]
    pub user_data_offset: i32,
    #[inspect(ignore)]
    pub transform_matrix: glam::Mat4,
    #[inspect(ignore)]
    pub inverse_matrix: glam::Mat4
}

impl Bone {
    /// Converts raw bone data into a more usable format by resolving the file offsets
    /// to proper node references. This ensures the editor knows which other files this one refers to.
    pub fn from_unresolved(
        bone: &UnresolvedBone,
        billboard_id: Option<IrNodeKey>,
        parent_id: Option<IrNodeKey>
    ) -> Self {
        Self {
            index: bone.index,
            id: bone.id,
            flags: bone.flags,
            billboard_setting: bone.billboard_setting,
            billboard_reference: billboard_id,
            scale: bone.scaling_vector,
            rotation: bone.rotation_vector,
            translation: bone.translation_vector,
            bounding_volume: bone.bounding_volume,
            parent: parent_id,
            user_data_offset: bone.user_data_offset,
            transform_matrix: bone.transform_matrix,
            inverse_matrix: bone.inverse_matrix
        }
    }
}

impl Visitable for Bone {
    fn accept(&self, node: VisitorCtxMeta<'_>, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_mdl()
            .map(|vis| vis.visit_bone(VisitorCtx::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut<'_>,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_mdl()
            .map(|vis| vis.visit_bone_mut(VisitorCtxMut::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}

/// Builds a nested tree of bones as nodes and returns the root node of the skeleton.
fn build_skeleton_tree(
    reader: &mut RefCursor<[u8]>,
    bones: Vec<LabeledBone>,
    arena: &IrArena
) -> SlipstreamResult<IrNodeKey> {
    /// The offset between the start of the bone and the bone's index.
    const BONE_INDEX_OFFSET: u64 = 3 * 4;

    // Create a virtual node for each of the bones.
    //
    // These will later be attached to each other to form a skeleton.
    let virtual_bones = bones
        .iter()
        .map(|bone| {
            arena.insert(IrNodeDescriptor {
                label: bone.label.clone(),
                ty: IrNodeType::MdlSection {
                    ty: MdlSectionType::Bones
                },
                parent: None,
                ..Default::default()
            })
        })
        .collect::<Vec<_>>();

    let mut found_root = None; // The bone that was determined to be the root of the skeleton.

    for (i, bone) in bones.iter().enumerate() {
        let curr_key = virtual_bones[i];

        // This bone has no parent, i.e it is the root.
        if bone.data.parent_offset == 0 {
            found_root = Some(i);

            arena.update(curr_key, |bone_node| {
                bone_node.ty = IrNodeType::MdlSection {
                    ty: MdlSectionType::Bones
                };
                bone_node.contents =
                    ContentSlot::eager(Box::new(Bone::from_unresolved(&bone.data, None, None)));
            });

            continue; // No parent
        }

        let parent_start = (bone.data.bone_start as i64)
            .checked_add(bone.data.parent_offset as i64)
            .and_then(|c| c.checked_add(BONE_INDEX_OFFSET as i64))
            .ok_or_else(|| {
                SlipstreamError::from(CorruptionError {
                    reason: format!("Parent bone pointer for bone `{i}` exceeds 64-bit integer limit (start: {}, parent_offset: {})", bone.data.bone_start, bone.data.parent_offset),
                    ..Default::default()
                })
            })?;

        reader.set_position(parent_start as u64);

        // Index into `virtual_bones` of the parent.
        let parent_index = reader.read_u32::<BigEndian>()?;
        if parent_index == i as u32 {
            // Ensure a bone is not its own parent.
            // This would cause cyclical references.

            return Err(InvalidInputError {
                reason: format!("bone `{}` is its own parent", bone.label),
                ..Default::default()
            }
            .into());
        }

        // Add this bone to its parent's children.
        if let Some(&parent_key) = virtual_bones.get(parent_index as usize) {
            try_unwrap!(
                arena.update(parent_key, |parent_node| {
                    parent_node.ty = IrNodeType::MdlSection {
                        ty: MdlSectionType::Bones
                    };
                    parent_node.children.push(curr_key);
                    parent_node.children.len() as u16 - 1
                }),
                "parent node {parent_key:?} was not found during skeleton resolution"
            )?;

            // Then update this bone's parent.
            arena.update(curr_key, |curr_node| {
                curr_node.parent = Some(parent_key);
                curr_node.contents = ContentSlot::eager(Box::new(Bone::from_unresolved(
                    &bone.data,
                    None,
                    Some(parent_key)
                )));
            });
        } else {
            tracing::error!(
                "Bone declares parent `{parent_index}`, but this parent does not exist."
            );
        }
    }

    let Some(root_index) = found_root else {
        return Err(InvalidInputError {
            reason: String::from("skeleton contained no root bone"),
            ..Default::default()
        }
        .into());
    };

    tracing::trace!("Skeleton constructed");

    let root = virtual_bones[root_index];

    Ok(root)
}

/// Deserializes the entire `Bones` section of an MDL file.
///
/// The parser automatically builds a proper file tree of bones that the outliner
/// can display. References between bones are resolved to use node IDs instead.
#[tracing::instrument(skip_all, fields(parent_id))]
pub fn deserialize_skeleton(
    reader: &mut RefCursor<[u8]>,
    parent_id: Option<IrNodeKey>,
    arena: &IrArena
) -> SlipstreamResult<IrNodeKey> {
    let section_index = IndexGroup::deserialize(reader)?;
    let mut bones = Vec::with_capacity(section_index.entries.len() - 1);

    for entry in &section_index.entries[1..] {
        let label = section_index.get_entry_name(reader, entry)?;
        let data_start = section_index.get_entry_data_start(entry)?;

        reader.set_position(data_start);

        let bone = UnresolvedBone::deserialize(reader)?;
        bones.push(LabeledBone { label, data: bone });
    }

    let node = build_skeleton_tree(reader, bones, arena)?;
    let root_key = arena.insert(IrNodeDescriptor {
        label: "Bones".to_owned(),
        ty: IrNodeType::MdlSection {
            ty: MdlSectionType::Bones
        },
        parent: parent_id,
        children: vec![node],
        ..Default::default()
    });

    Ok(root_key)
}
