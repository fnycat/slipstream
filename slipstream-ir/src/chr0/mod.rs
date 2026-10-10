mod frame;

use std::ops::ControlFlow;

pub use frame::*;

use bitfield_struct::{bitenum, bitfield};
use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::{Inspect, inspect_bitfield};
use slipstream_shared::{
    cursor::RefCursor,
    error::{CorruptionError, SlipstreamError, SlipstreamResult},
    verify,
};

use crate::{
    brres::{self, BFile, BFileHeader, BFileType},
    index::IndexGroup,
    node::{
        arena::{IrArena, IrNodeDescriptor, IrNodeKey},
        node::{ContentSlot, IrNodeType},
    },
    visitor::{
        Visitable, Visitor, VisitorContext, VisitorContextMut, VisitorContextNode,
        VisitorContextNodeMut,
    },
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Inspect)]
pub enum AnimationPolicy {
    OneTime,
    Loop,
}

impl TryFrom<u32> for AnimationPolicy {
    type Error = SlipstreamError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::OneTime,
            0x01 => Self::Loop,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid animation policy: {value} (expected 0 or 1)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

impl AnimationPolicy {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let policy = reader.read_u32::<BigEndian>()?;
        Self::try_from(policy)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Inspect)]
#[repr(u32)]
pub enum ScalingRule {
    Standard,
    #[inspect(rename = "Autodesk Softimage")]
    Softimage,
    #[inspect(rename = "Autodesk Maya")]
    Maya,
}

impl TryFrom<u32> for ScalingRule {
    type Error = SlipstreamError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::Standard,
            0x01 => Self::Softimage,
            0x02 => Self::Maya,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid scaling rule: {} (expected 0, 1 or 2)", value),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

impl ScalingRule {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let rule = reader.read_u32::<BigEndian>()?;
        Self::try_from(rule)
    }
}

/// The format used to store the animation. This is used for translation and scale animations.
/// Rotations used [`AnimationFormat3`] instead.
///
/// This affects the quality and behavior of the animation.
#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Inspect)]
#[repr(u8)]
pub enum AnimationFormat2 {
    /// The bone is kept in a fixed place. Fixed animations are simply a single value indicating where to
    /// put the bone.
    #[inspect(
        tooltip = "The bone is not animated. It is kept fixed in place and this animation only specifies that position."
    )]
    Fixed = 0b000,
    /// A 4-byte format where the animation is evaluated by smoothly interpolating between
    /// a few explicitly defined keyframes. Each defined keyframe specifies its tangent to create
    /// smooth curves between frames.
    #[inspect(
        tooltip = "The animation is evaluated by smoothly interpolating between \
        a few explicitly defined keyframes. Each defined keyframe specifies its tangent to create \
        smooth curves between frames. \
        \
        Each explicitly set keyframe is 4 bytes in size."
    )]
    Interpolated4 = 0b001,
    /// A 6-byte format where the animation is evaluated by smoothly interpolating between
    /// a few explicitly defined keyframes. Each defined keyframe specifies its tangent to create
    /// smooth curves between frames.
    #[inspect(
        tooltip = "The animation is evaluated by smoothly interpolating between \
        a few explicitly defined keyframes. Each defined keyframe specifies its tangent to create \
        smooth curves between frames. \
        \
        Each explicitly set keyframe is 6 bytes in size."
    )]
    Interpolated6 = 0b010,
    /// A 12-byte format where the animation is evaluated by smoothly interpolating between
    /// a few explicitly defined keyframes. Each defined keyframe specifies its tangent to create
    /// smooth curves between frames.
    #[inspect(
        tooltip = "The animation is evaluated by smoothly interpolating between \
        a few explicitly defined keyframes. Each defined keyframe specifies its tangent to create \
        smooth curves between frames. \
        \
        Each explicitly set keyframe is 12 bytes in size."
    )]
    Interpolated12 = 0b011,
    /// A fallback for [`bitenum`], this variant should never be used.
    #[inspect(ignore)]
    #[fallback]
    Invalid,
}

impl AnimationFormat2 {
    /// Converts this format to an extended 3-bit format.
    pub fn extend(self) -> AnimationFormat3 {
        match self {
            Self::Fixed => AnimationFormat3::Fixed,
            Self::Interpolated4 => AnimationFormat3::Interpolated4,
            Self::Interpolated6 => AnimationFormat3::Interpolated6,
            Self::Interpolated12 => AnimationFormat3::Interpolated12,
            Self::Invalid => AnimationFormat3::Invalid,
        }
    }
}

/// The format used to store the animation. This is an extended version for the rotation format.
/// As rotations support two more formats.
///
/// This affects the quality and behavior of the animation.
#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Inspect)]
#[repr(u8)]
pub enum AnimationFormat3 {
    /// The bone is kept in a fixed place. Fixed animations are simply a single value indicating where to
    /// put the bone.
    #[inspect(
        tooltip = "The bone is not animated. It is kept fixed in place and this animation only specifies that position."
    )]
    Fixed = 0b000,
    /// A 4-byte format where the animation is evaluated by smoothly interpolating between
    /// a few explicitly defined keyframes. Each defined keyframe specifies its tangent to create
    /// smooth curves between frames.
    #[inspect(
        tooltip = "The animation is evaluated by smoothly interpolating between \
        a few explicitly defined keyframes. Each defined keyframe specifies its tangent to create \
        smooth curves between frames. \
        \
        Each explicitly set keyframe is 4 bytes in size."
    )]
    Interpolated4 = 0b001,
    /// A 6-byte format where the animation is evaluated by smoothly interpolating between
    /// a few explicitly defined keyframes. Each defined keyframe specifies its tangent to create
    /// smooth curves between frames.
    #[inspect(
        tooltip = "The animation is evaluated by smoothly interpolating between \
        a few explicitly defined keyframes. Each defined keyframe specifies its tangent to create \
        smooth curves between frames. \
        \
        Each explicitly set keyframe is 6 bytes in size."
    )]
    Interpolated6 = 0b010,
    /// A 12-byte format where the animation is evaluated by smoothly interpolating between
    /// a few explicitly defined keyframes. Each defined keyframe specifies its tangent to create
    /// smooth curves between frames.
    #[inspect(
        tooltip = "The animation is evaluated by smoothly interpolating between \
        a few explicitly defined keyframes. Each defined keyframe specifies its tangent to create \
        smooth curves between frames. \
        \
        Each explicitly set keyframe is 12 bytes in size."
    )]
    Interpolated12 = 0b011,
    /// Specifies every single keyframe explicitly, but allows for more erratic information instead of the smoothly
    /// interpolated animations of interpolation formats.
    ///
    /// The frames are 1 byte in size.
    #[inspect(
        tooltip = "Specifies every single keyframe explicitly, but allows for more erratic information instead of the smoothly \
        interpolated animations of interpolation formats. \
        \
        The keyframes are 1 byte in size."
    )]
    Linear1 = 0b100,
    /// Specifies every single keyframe explicitly, but allows for more erratic information instead of the smoothly
    /// interpolated animations of interpolation formats.
    ///
    /// The frames are 4 bytes in size.
    #[inspect(
        tooltip = "Specifies every single keyframe explicitly, but allows for more erratic information instead of the smoothly \
        interpolated animations of interpolation formats. \
        \
        The keyframes are 4 bytes in size."
    )]
    Linear4 = 0b110,
    /// A fallback for [`bitenum`], this variant should never be used.
    #[inspect(ignore)]
    #[fallback]
    Invalid,
}

impl AnimationFormat3 {
    /// Whether this format is a linear format.
    pub fn is_linear(&self) -> bool {
        const LINEAR_MASK: u8 = 0b100;
        (*self as u8) & LINEAR_MASK != 0
    }
}

#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(summary = "AnimationCode::summary")]
pub struct AnimationCode {
    #[bits(1)]
    _unused: bool,
    pub use_identity: bool,
    pub rotation_translation_isotropic: bool,
    pub scale_isotropic: bool,
    pub scale_uniform: bool,
    pub rotation_isotropic: bool,
    pub translation_isotropic: bool,
    pub use_model_scale: bool,
    pub use_model_rotation: bool,
    pub use_model_translation: bool,
    pub apply_scale_compensate: bool,
    pub apply_child_scale_compensate: bool,
    pub disable_classic_scale: bool,
    pub scale_x_fixed: bool,
    pub scale_y_fixed: bool,
    pub scale_z_fixed: bool,
    pub rotation_x_fixed: bool,
    pub rotation_y_fixed: bool,
    pub rotation_z_fixed: bool,
    pub x_fixed: bool,
    pub y_fixed: bool,
    pub z_fixed: bool,
    pub has_scale: bool,
    pub has_rotation: bool,
    pub has_translation: bool,
    #[bits(2)]
    pub scale_format: AnimationFormat2,
    #[bits(3)]
    pub rotation_format: AnimationFormat3,
    #[bits(2)]
    pub translation_format: AnimationFormat2,
}

impl AnimationCode {
    #[inline]
    pub fn summary(&self) -> String {
        format!("{:#08x}", self.into_bits())
    }

    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }
}

/// The type of animation that is applied to the bone.
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    /// The bone stays in place throughout the entire animation.
    Fixed(f32),
    /// The bone is animated.
    Animated(AnimationType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationType {
    Interpolated4(I4Animation),
    Interpolated6(I6Animation),
    Interpolated12(I12Animation),
    Linear1(L1Animation),
    Linear4(L4Animation),
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct ComponentData {
    #[inspect(ignore)]
    pub x: ComponentType,
    #[inspect(ignore)]
    pub y: ComponentType,
    #[inspect(ignore)]
    pub z: ComponentType,
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct AnimationData {
    pub translation: Option<ComponentData>,
    pub rotation: Option<ComponentData>,
    pub scale: Option<ComponentData>,
}

impl AnimationData {
    /// Deserializes the current keyframe.
    fn deserialize_keyframe(
        reader: &mut RefCursor<[u8]>,
        bone_data_start: u64,
        header_frame_count: u16,
        format: AnimationFormat3,
    ) -> SlipstreamResult<AnimationType> {
        let orig_position = reader.position();

        let frame_offset = reader.read_i32::<BigEndian>()? as i64;
        let frame_start = bone_data_start as i64 + frame_offset;
        reader.set_position(frame_start as u64);

        let frames = match format {
            AnimationFormat3::Interpolated4 => {
                AnimationType::Interpolated4(I4Animation::deserialize(reader, header_frame_count)?)
            }
            AnimationFormat3::Interpolated6 => {
                AnimationType::Interpolated6(I6Animation::deserialize(reader, header_frame_count)?)
            }
            AnimationFormat3::Interpolated12 => AnimationType::Interpolated12(
                I12Animation::deserialize(reader, header_frame_count)?,
            ),
            AnimationFormat3::Linear1 => {
                // Does Brawlcrate simply just display them differently?
                tracing::error!("FIXME: LINEAR1 ANIMATIONS DO NOT WORK PROPERLY RIGHT NOW");
                AnimationType::Linear1(L1Animation::deserialize(reader, header_frame_count)?)
            }
            _ => todo!("animation frame format {format:?}"),
        };

        reader.set_position(orig_position + 4);
        Ok(frames)
    }

    /// Returns the scale data of the current keyframe.
    fn deserialize_scale(
        reader: &mut RefCursor<[u8]>,
        bone_data_start: u64,
        frame_count: u16,
        anim_ty_code: &AnimationCode,
    ) -> SlipstreamResult<ComponentData> {
        tracing::trace!(
            "Reading scale animations (iso: {}, x fixed: {}, y fixed: {}, z fixed: {})",
            anim_ty_code.scale_isotropic(),
            anim_ty_code.scale_x_fixed(),
            anim_ty_code.scale_y_fixed(),
            anim_ty_code.scale_z_fixed()
        );

        if anim_ty_code.scale_isotropic() {
            let iso_scale;

            // Only one piece of data is stored, rather than for each component
            if anim_ty_code.scale_x_fixed() {
                iso_scale = ComponentType::Fixed(reader.read_f32::<BigEndian>()?);
            } else {
                let frame = Self::deserialize_keyframe(
                    reader,
                    bone_data_start,
                    frame_count,
                    anim_ty_code.scale_format().extend(),
                )?;
                iso_scale = ComponentType::Animated(frame);
            }

            Ok(ComponentData {
                x: iso_scale.clone(),
                y: iso_scale.clone(),
                z: iso_scale,
            })
        } else {
            let x_scale = if anim_ty_code.scale_x_fixed() {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                let frame = Self::deserialize_keyframe(
                    reader,
                    bone_data_start,
                    frame_count,
                    anim_ty_code.scale_format().extend(),
                )?;
                ComponentType::Animated(frame)
            };

            let y_scale = if anim_ty_code.scale_y_fixed() {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                let frame = Self::deserialize_keyframe(
                    reader,
                    bone_data_start,
                    frame_count,
                    anim_ty_code.scale_format().extend(),
                )?;
                ComponentType::Animated(frame)
            };

            let z_scale = if anim_ty_code.scale_z_fixed() {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                let frame = Self::deserialize_keyframe(
                    reader,
                    bone_data_start,
                    frame_count,
                    anim_ty_code.scale_format().extend(),
                )?;
                ComponentType::Animated(frame)
            };

            Ok(ComponentData {
                x: x_scale,
                y: y_scale,
                z: z_scale,
            })
        }
    }

    /// Reads the rotation data of the current keyframe.
    fn deserialize_rotation(
        reader: &mut RefCursor<[u8]>,
        bone_data_start: u64,
        frame_count: u16,
        anim_code: &AnimationCode,
    ) -> SlipstreamResult<ComponentData> {
        tracing::trace!(
            "Reading rotation animations (iso: {}, x fixed: {}, y fixed: {}, z fixed: {})",
            anim_code.rotation_isotropic(),
            anim_code.rotation_x_fixed(),
            anim_code.rotation_y_fixed(),
            anim_code.rotation_z_fixed()
        );

        if anim_code.rotation_isotropic() {
            let iso_rot = if anim_code.rotation_x_fixed() {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                let frame = Self::deserialize_keyframe(
                    reader,
                    bone_data_start,
                    frame_count,
                    anim_code.rotation_format(),
                )?;
                ComponentType::Animated(frame)
            };

            Ok(ComponentData {
                x: iso_rot.clone(),
                y: iso_rot.clone(),
                z: iso_rot,
            })
        } else {
            let x_rot = if anim_code.rotation_x_fixed() {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                let frame = Self::deserialize_keyframe(
                    reader,
                    bone_data_start,
                    frame_count,
                    anim_code.rotation_format(),
                )?;
                ComponentType::Animated(frame)
            };

            let y_rot = if anim_code.rotation_y_fixed() {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                let frame = Self::deserialize_keyframe(
                    reader,
                    bone_data_start,
                    frame_count,
                    anim_code.rotation_format(),
                )?;
                ComponentType::Animated(frame)
            };

            let z_rot = if anim_code.rotation_z_fixed() {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                let frame = Self::deserialize_keyframe(
                    reader,
                    bone_data_start,
                    frame_count,
                    anim_code.rotation_format(),
                )?;
                ComponentType::Animated(frame)
            };

            Ok(ComponentData {
                x: x_rot,
                y: y_rot,
                z: z_rot,
            })
        }
    }

    /// Reads the translation data of the current keyframe.
    fn deserialize_translation(
        reader: &mut RefCursor<[u8]>,
        bone_data_start: u64,
        frame_count: u16,
        anim_code: &AnimationCode,
    ) -> SlipstreamResult<ComponentData> {
        tracing::trace!(
            "Reading translation animations (iso: {}, x fixed: {}, y fixed: {}, z fixed: {})",
            anim_code.translation_isotropic(),
            anim_code.x_fixed(),
            anim_code.y_fixed(),
            anim_code.z_fixed()
        );

        if anim_code.translation_isotropic() {
            let iso_trans = if anim_code.x_fixed() {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                ComponentType::Animated(Self::deserialize_keyframe(
                    reader,
                    bone_data_start,
                    frame_count,
                    anim_code.translation_format().extend(),
                )?)
            };

            Ok(ComponentData {
                x: iso_trans.clone(),
                y: iso_trans.clone(),
                z: iso_trans,
            })
        } else {
            let trans_x = if anim_code.x_fixed() {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                ComponentType::Animated(Self::deserialize_keyframe(
                    reader,
                    bone_data_start,
                    frame_count,
                    anim_code.translation_format().extend(),
                )?)
            };

            let trans_y = if anim_code.y_fixed() {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                ComponentType::Animated(Self::deserialize_keyframe(
                    reader,
                    bone_data_start,
                    frame_count,
                    anim_code.translation_format().extend(),
                )?)
            };

            let trans_z = if anim_code.z_fixed() {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                ComponentType::Animated(Self::deserialize_keyframe(
                    reader,
                    bone_data_start,
                    frame_count,
                    anim_code.translation_format().extend(),
                )?)
            };

            Ok(ComponentData {
                x: trans_x,
                y: trans_y,
                z: trans_z,
            })
        }
    }

    /// Deserializes all (translation, rotation and scale) components of a single keyframe.
    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        bone_data_start: u64,
        frame_count: u16,
        anim_code: &AnimationCode,
    ) -> SlipstreamResult<Self> {
        let scale = if anim_code.has_scale() {
            Some(Self::deserialize_scale(
                reader,
                bone_data_start,
                frame_count,
                anim_code,
            )?)
        } else {
            None
        };

        let rotation = if anim_code.has_rotation() {
            Some(Self::deserialize_rotation(
                reader,
                bone_data_start,
                frame_count,
                anim_code,
            )?)
        } else {
            None
        };

        let translation = if anim_code.has_translation() {
            Some(Self::deserialize_translation(
                reader,
                bone_data_start,
                frame_count,
                anim_code,
            )?)
        } else {
            None
        };

        // If a component is fixed, the current float is simply the value for the bone.
        Ok(AnimationData {
            scale,
            rotation,
            translation,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct SkeletalAnimation {
    #[inspect(rename = "Animation Settings")]
    pub anim_code: AnimationCode,
    #[inspect(rename = "Animation Data")]
    pub anim_data: AnimationData,
}

impl SkeletalAnimation {
    #[tracing::instrument(skip(reader, frame_count))]
    pub fn deserialize(reader: &mut RefCursor<[u8]>, frame_count: u16) -> SlipstreamResult<Self> {
        let anim_data_start = reader.position();

        // Points to the same string as the file name in the index group entry,
        // so we don't need it.
        let _bone_name_offset = reader.read_u32::<BigEndian>()?;
        let anim_code = AnimationCode::deserialize(reader)?;
        let anim_data =
            AnimationData::deserialize(reader, anim_data_start, frame_count, &anim_code)?;

        Ok(Self {
            anim_code,
            // anim_flags,
            anim_data,
        })
    }
}

impl Visitable for SkeletalAnimation {
    fn accept(&self, node: VisitorContextNode, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor.visit_skeletal_animation(VisitorContext::new(node, self))
    }

    fn accept_mut(
        &mut self,
        node: VisitorContextNodeMut,
        visitor: &mut dyn Visitor,
    ) -> ControlFlow<()> {
        visitor.visit_skeletal_animation_mut(VisitorContextMut::new(node, self))
    }
}

pub const CHR0_MAGIC: [u8; 4] = [0x43, 0x48, 0x52, 0x30];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SectionType {
    Animations = 0,
    UserData = 1,
}

impl TryFrom<usize> for SectionType {
    type Error = SlipstreamError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Animations,
            1 => Self::UserData,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid CHR0 section type: {value} (expected 0 or 1)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Inspect)]
pub struct Chr0Root {
    /// The amount of animation frames stored in this file.
    pub frame_count: u16,
    #[inspect(rename = "Keyframe Count")]
    pub anim_data_count: u16,
    /// Whether the animation loops or is a one time animation.
    #[inspect(rename = "Animation Policy")]
    pub anim_policy: AnimationPolicy,
    pub scaling_rule: ScalingRule,
}

impl Chr0Root {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let frame_count = reader.read_u16::<BigEndian>()?;
        let anim_data_count = reader.read_u16::<BigEndian>()?;
        let anim_policy = AnimationPolicy::deserialize(reader)?;
        let scaling_rule = ScalingRule::deserialize(reader)?;

        Ok(Self {
            frame_count,
            anim_data_count,
            anim_policy,
            scaling_rule,
        })
    }
}

impl Visitable for Chr0Root {
    fn accept(&self, node: VisitorContextNode, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor.visit_chr0(VisitorContext::new(node, self))
    }

    fn accept_mut(
        &mut self,
        node: VisitorContextNodeMut,
        visitor: &mut dyn Visitor,
    ) -> ControlFlow<()> {
        visitor.visit_chr0_mut(VisitorContextMut::new(node, self))
    }
}

#[tracing::instrument(skip_all, fields(name, parent_id))]
pub fn deserialize(
    reader: &mut RefCursor<[u8]>,
    parent_id: IrNodeKey,
    arena: &IrArena,
    name: String,
) -> SlipstreamResult<IrNodeKey> {
    let subfile_header = BFileHeader::deserialize(reader, BFileType::Chr0)?;
    reader.set_position(reader.position() + 4); // there are 4 bytes of padding between the headers

    // CHR0 v3 has a single section. Only the animation data appears here.
    // CHR0 v5 instead has two where presumably the second section is user data.

    let expected_sections = brres::get_section_count(BFileType::Chr0, subfile_header.version)?;
    verify!(
        subfile_header.offsets.len() == expected_sections,
        "CHR0 section count ({}) did not match expected section count ({expected_sections})",
        subfile_header.offsets.len()
    );

    let chr0_header = Chr0Root::deserialize(reader)?;

    let chr0_root_key = arena.reserve_key();

    let mut files = Vec::new();
    for (i, &section_offset) in subfile_header.offsets.iter().enumerate() {
        if section_offset == 0 {
            // Section does not exist.
            continue;
        }

        reader.set_position(subfile_header.get_section_start(i)?);

        let section_ty = SectionType::try_from(i)?;
        match section_ty {
            SectionType::Animations => {
                // Instead of creating subfolders for the different sections, like in MDL0,
                // we instead just append it the root. This is because there are only two possible sections,
                // of which user data usually doesn't exist.

                let index = IndexGroup::deserialize(reader)?;
                for entry in &index.entries[1..] {
                    let name = index.get_entry_name(reader, entry)?;
                    let data_start = index.get_entry_data_start(entry);

                    reader.set_position(data_start);

                    let anim = SkeletalAnimation::deserialize(reader, chr0_header.frame_count)?;

                    let anim_file = arena.insert(IrNodeDescriptor {
                        label: name,
                        ty: IrNodeType::SkeletalAnimation,
                        parent: Some(chr0_root_key),
                        children: Vec::new(),
                        contents: ContentSlot::eager(Box::new(anim)),
                    });
                    files.push(anim_file);
                }
            }
            SectionType::UserData => todo!("CHR0 user data"),
        };
    }

    arena.insert_at(
        chr0_root_key,
        IrNodeDescriptor {
            label: name,
            ty: IrNodeType::Chr0Root,
            parent: Some(parent_id),
            children: files,
            contents: ContentSlot::eager(Box::new(chr0_header)),
        },
    );

    Ok(chr0_root_key)
}
