use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::{Inspect, inspect_bitfield};
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::brres::{SubfileHeader, SubfileType};
use crate::index::IndexGroup;
use crate::node::arena::{IrArena, IrNodeDescriptor, IrNodeKey};
use crate::node::node::{ContentSlot, IrNodeType};
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

pub const CLR_MAGIC: [u8; 4] = [0x43, 0x4c, 0x52, 0x30]; // "CLR0"

#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(summary = AnimationFlag::summary)]
pub struct AnimationFlag {
    #[inspect(rename = "Has Material 1")]
    pub has_material0: bool,
    #[inspect(rename = "Material 1 Constant")]
    pub constant_material0: bool,
    #[inspect(rename = "Has Material 2")]
    pub has_material1: bool,
    #[inspect(rename = "Material 2 Constant")]
    pub constant_material1: bool,
    #[inspect(rename = "Has Ambient 1")]
    pub has_ambient0: bool,
    #[inspect(rename = "Ambient Constant 1")]
    pub constant_ambient0: bool,
    #[inspect(rename = "Has Ambient 2")]
    pub has_ambient1: bool,
    #[inspect(rename = "Ambient Constant 2")]
    pub constant_ambient1: bool,
    #[inspect(rename = "Has TEV Register 1")]
    pub has_tev_register0: bool,
    #[inspect(rename = "TEV Register 1 Constant")]
    pub constant_tev_register0: bool,
    #[inspect(rename = "Has TEV Register 2")]
    pub has_tev_register1: bool,
    #[inspect(rename = "TEV Register 2 Constant")]
    pub constant_tev_register1: bool,
    #[inspect(rename = "Has TEV Register 3")]
    pub has_tev_register2: bool,
    #[inspect(rename = "TEV Register 3 Constant")]
    pub constant_tev_register2: bool,
    #[inspect(rename = "Has TEV Register 4")]
    pub has_tev_register3: bool,
    #[inspect(rename = "TEV Register 4 Constant")]
    pub constant_tev_register3: bool,
    #[bits(16)]
    _unused: u16
}

impl AnimationFlag {
    pub fn summary(&self) -> String {
        format!("{:#08x}", self.into_bits())
    }

    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Inspect)]
pub struct MaterialData {
    pub target_name_offset: u32,
    pub animation_flag: AnimationFlag
}

impl MaterialData {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let target_name_offset = reader.read_u32::<BigEndian>()?;
        let animation_flag = AnimationFlag::deserialize(reader)?;

        Ok(Self {
            target_name_offset,
            animation_flag
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Inspect)]
pub struct MaterialAnimationDataValue {
    pub rgba_mask: u32,
    pub rgba_offset: u32
}

impl MaterialAnimationDataValue {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let rgba_mask = reader.read_u32::<BigEndian>()?;
        let rgba_offset = reader.read_u32::<BigEndian>()?;

        Ok(Self {
            rgba_mask,
            rgba_offset
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Inspect)]
pub struct MaterialAnimationData {
    #[inspect(rename = "Light Channel Material 1")]
    pub light_channel_material0: MaterialAnimationDataValue,
    #[inspect(rename = "Light Channel Material 2")]
    pub light_channel_material1: MaterialAnimationDataValue,
    #[inspect(rename = "Light Channel Ambient 1")]
    pub light_channel_ambient0: MaterialAnimationDataValue,
    #[inspect(rename = "Light Channel Ambient 2")]
    pub light_channel_ambient1: MaterialAnimationDataValue,
    #[inspect(rename = "Color Register 1")]
    pub color_register0: MaterialAnimationDataValue,
    #[inspect(rename = "Color Register 2")]
    pub color_register1: MaterialAnimationDataValue,
    #[inspect(rename = "Color Register 3")]
    pub color_register2: MaterialAnimationDataValue,
    #[inspect(rename = "Constant Color Register 1")]
    pub constant_color_register0: MaterialAnimationDataValue,
    #[inspect(rename = "Constant Color Register 2")]
    pub constant_color_register1: MaterialAnimationDataValue,
    #[inspect(rename = "Constant Color Register 3")]
    pub constant_color_register2: MaterialAnimationDataValue,
    #[inspect(rename = "Constant Color Register 4")]
    pub constant_color_register3: MaterialAnimationDataValue
}

impl MaterialAnimationData {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let light_channel_material0 = MaterialAnimationDataValue::deserialize(reader)?;
        let light_channel_material1 = MaterialAnimationDataValue::deserialize(reader)?;
        let light_channel_ambient0 = MaterialAnimationDataValue::deserialize(reader)?;
        let light_channel_ambient1 = MaterialAnimationDataValue::deserialize(reader)?;
        let color_register0 = MaterialAnimationDataValue::deserialize(reader)?;
        let color_register1 = MaterialAnimationDataValue::deserialize(reader)?;
        let color_register2 = MaterialAnimationDataValue::deserialize(reader)?;
        let constant_color_register0 = MaterialAnimationDataValue::deserialize(reader)?;
        let constant_color_register1 = MaterialAnimationDataValue::deserialize(reader)?;
        let constant_color_register2 = MaterialAnimationDataValue::deserialize(reader)?;
        let constant_color_register3 = MaterialAnimationDataValue::deserialize(reader)?;

        Ok(Self {
            light_channel_material0,
            light_channel_material1,
            light_channel_ambient0,
            light_channel_ambient1,
            color_register0,
            color_register1,
            color_register2,
            constant_color_register0,
            constant_color_register1,
            constant_color_register2,
            constant_color_register3
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Inspect)]
pub struct ColorAnimation {
    pub data: MaterialData,
    pub anim_data: MaterialAnimationData
}

impl ColorAnimation {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let data = MaterialData::deserialize(reader)?;
        let anim_data = MaterialAnimationData::deserialize(reader)?;

        Ok(Self { data, anim_data })
    }
}

impl Visitable for ColorAnimation {
    fn accept(&self, node: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor.visit_color_animation(VisitorCtx::new(node, self))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor.visit_color_animation_mut(VisitorCtxMut::new(node, self))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Inspect)]
pub struct ClrHeader {
    pub brres_header: SubfileHeader,
    pub start_position: u64,
    pub frame_count: u16,
    pub animated_materials: u16,
    pub looping: bool
}

impl ClrHeader {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let subfile_header = SubfileHeader::deserialize(reader, SubfileType::Clr)?;

        let start = reader.position();
        let _unused = reader.read_u32::<BigEndian>()?;
        let frame_count = reader.read_u16::<BigEndian>()?;
        let material_count = reader.read_u16::<BigEndian>()?;
        let looping = reader.read_u32::<BigEndian>()? != 0;

        Ok(Self {
            brres_header: subfile_header,
            start_position: start,
            frame_count,
            animated_materials: material_count,
            looping
        })
    }
}

impl Visitable for ClrHeader {
    fn accept(&self, node: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor.visit_clr(VisitorCtx::new(node, self))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor.visit_clr_mut(VisitorCtxMut::new(node, self))
    }
}

#[tracing::instrument(skip_all, fields(name, parent_id))]
pub fn deserialize(
    reader: &mut RefCursor<[u8]>,
    parent_id: Option<IrNodeKey>,
    arena: &IrArena,
    name: String
) -> SlipstreamResult<IrNodeKey> {
    let clr_header = ClrHeader::deserialize(reader)?;

    let clr_key = arena.reserve_key();
    let index = IndexGroup::deserialize(reader)?;

    tracing::debug!("Deserializing {} color animations", index.entries.len() - 1);

    let mut children = Vec::with_capacity(index.entries.len() - 1);
    for entry in &index.entries {
        let name = index.get_entry_name(reader, entry)?;
        let data_start = index.get_entry_data_start(entry)?;

        tracing::debug!("Reading color animation `{name}`");

        reader.set_position(data_start);

        let material_data = MaterialData::deserialize(reader)?;
        let material_anim_data = MaterialAnimationData::deserialize(reader)?;

        let key = arena.insert(IrNodeDescriptor {
            label: name,
            parent: Some(clr_key),
            ty: IrNodeType::ColorAnimation,
            contents: ContentSlot::eager(Box::new(ColorAnimation {
                data: material_data,
                anim_data: material_anim_data
            })),
            ..Default::default()
        });

        children.push(key);
    }

    arena.insert_at(
        clr_key,
        IrNodeDescriptor {
            label: name,
            ty: IrNodeType::Clr0Root,
            parent: parent_id,
            children,
            contents: ContentSlot::eager(Box::new(clr_header))
        }
    );

    Ok(clr_key)
}
