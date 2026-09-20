use byteorder::{BigEndian, ReadBytesExt};

use crate::error::{CorruptionError, EditorResult};
use crate::format::brres::IndexGroup;
use crate::format::encoding::Deserialize;
use crate::node::defer::Deferred;
use crate::node::node::{VirtualNode, VirtualNodeBody, VirtualNodeKind};
use crate::node::refs::{VirtualNodeId, VirtualNodeMap, VirtualNodeRef};
use crate::{format::mdl0::SectionDeserialize, shared::util::RefCursor};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapNode {
    pub bone_index: u16,
    pub parent_matrix_index: u16,
}

impl MapNode {
    pub const OPCODE: u8 = 0x02;
}

impl Deserialize for MapNode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let bone_index = reader.read_u16::<BigEndian>()?;
        let parent_matrix_index = reader.read_u16::<BigEndian>()?;

        Ok(Self {
            bone_index,
            parent_matrix_index,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Weight {
    pub bone_id: u16,
    pub weight: f32,
}

impl Deserialize for Weight {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let bone_id = reader.read_u16::<BigEndian>()?;
        let weight = reader.read_f32::<BigEndian>()?;

        Ok(Self { bone_id, weight })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Weights {
    pub weight_id: u16,
    pub weights: Vec<Weight>,
}

impl Weights {
    pub const OPCODE: u8 = 0x03;
}

impl Deserialize for Weights {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let weight_id = reader.read_u16::<BigEndian>()?;
        let weight_count = reader.read_u8()?;

        let mut weights = Vec::with_capacity(weight_count as usize);
        for _ in 0..weight_count {
            weights.push(Weight::deserialize(reader)?);
        }

        Ok(Self { weight_id, weights })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawPolygon {
    pub material_index: u16,
    pub object_index: u16,
    pub bone_index: u16,
    pub priority: u8,
}

impl DrawPolygon {
    pub const OPCODE: u8 = 0x04;
}

impl Deserialize for DrawPolygon {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let material_index = reader.read_u16::<BigEndian>()?;
        let object_index = reader.read_u16::<BigEndian>()?;
        let bone_index = reader.read_u16::<BigEndian>()?;
        let priority = reader.read_u8()?;

        // Skip over empty bytes
        reader.set_position(reader.position() + 2);

        Ok(Self {
            material_index,
            object_index,
            bone_index,
            priority,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightIndex {
    pub matrix_id: u16,
    pub weight_index: u16,
}

impl WeightIndex {
    pub const OPCODE: u8 = 0x05;
}

impl Deserialize for WeightIndex {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let matrix_id = reader.read_u16::<BigEndian>()?;
        let weight_index = reader.read_u16::<BigEndian>()?;

        // Skip over empty bytes
        reader.set_position(reader.position() + 5);

        Ok(Self {
            matrix_id,
            weight_index,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateMatrix {
    pub dest: u16,
    pub src: u16,
}

impl DuplicateMatrix {
    pub const OPCODE: u8 = 0x06;
}

impl Deserialize for DuplicateMatrix {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let dest = reader.read_u16::<BigEndian>()?;
        let src = reader.read_u16::<BigEndian>()?;

        // Skip over empty bytes
        reader.set_position(reader.position() + 5);

        Ok(Self { dest, src })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawCommand {
    MapNode(MapNode),
    Weights(Weights),
    DrawPolygon(DrawPolygon),
    WeightIndex(WeightIndex),
    DuplicateMatrix(DuplicateMatrix),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bytecode {
    commands: Vec<DrawCommand>,
}

impl Bytecode {
    pub const NOPCODE: u8 = 0x00;
    pub const END_OPCODE: u8 = 0x01;
}

impl Deserialize for Bytecode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let mut commands = Vec::new();

        let mut opcode = reader.read_u8()?;
        while opcode != Self::END_OPCODE {
            let command = match opcode {
                Self::NOPCODE => {
                    reader.set_position(reader.position() + 8);
                    opcode = reader.read_u8()?;

                    continue;
                } // This command is empty
                MapNode::OPCODE => DrawCommand::MapNode(MapNode::deserialize(reader)?),
                Weights::OPCODE => DrawCommand::Weights(Weights::deserialize(reader)?),
                DrawPolygon::OPCODE => DrawCommand::DrawPolygon(DrawPolygon::deserialize(reader)?),
                WeightIndex::OPCODE => DrawCommand::WeightIndex(WeightIndex::deserialize(reader)?),
                DuplicateMatrix::OPCODE => {
                    DrawCommand::DuplicateMatrix(DuplicateMatrix::deserialize(reader)?)
                }
                _ => {
                    return Err(CorruptionError {
                        reason: format!("invalid draw list opcode: {opcode:#04x}"),
                        location: Some(reader.position()),
                        ..Default::default()
                    }
                    .into());
                }
            };

            commands.push(command);
            opcode = reader.read_u8()?;
        }

        Ok(Self { commands })
    }
}

pub fn deserialize_virtual(
    reader: &mut RefCursor<[u8]>,
    parent_id: VirtualNodeId,
    node_map: &VirtualNodeMap,
) -> EditorResult<VirtualNodeBody> {
    let section_index = IndexGroup::deserialize(reader)?;

    let mut children = Vec::with_capacity(section_index.entries.len() - 1);
    for entry in &section_index.entries[1..] {
        let name = section_index.get_entry_name(reader, entry)?;
        let data_start = section_index.get_entry_data_start(entry);

        reader.set_position(data_start as u64);
        let draw_list = Bytecode::deserialize(reader)?;

        let id = node_map.next_id();
        let node = VirtualNode {
            label: name,
            id,
            kind: VirtualNodeKind::Bytecode,
            parent: Some(parent_id),
            body: Deferred::evaluated(VirtualNodeBody {
                children: Vec::new(),
                inspectable: Some(Box::new(draw_list)),
            }),
        };

        node_map.insert(id, node);
        children.push(id);
    }

    dbg!(&children);

    Ok(VirtualNodeBody {
        children,
        inspectable: None,
    })
}
