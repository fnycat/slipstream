use byteorder::{BigEndian, ReadBytesExt};

use crate::error::{CorruptionError, EditorResult};
use crate::format::brres::IndexGroup;
use crate::node::defer::Deferred;
use crate::node::node::{VirtualNode, VirtualNodeBody, VirtualNodeKind};
use crate::node::refs::{VirtualNodeId, VirtualNodeMap};
use crate::{
    format::{
        encoding::Deserialize,
        mdl0::util::{VectorFormat, deserialize_vector_data},
    },
    shared::util::RefCursor,
};

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

impl NormalData {
    pub fn len(&self) -> usize {
        match self {
            Self::Normal(x) => x.len(),
            Self::All(x) => x.len(),
            Self::Any(x) => x.len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Normals {
    pub index: u32,
    pub format: VectorFormat,
    pub divisor: u8,
    pub stride: u8,
    pub normals: NormalData,
}

pub fn deserialize_virtual(
    reader: &mut RefCursor<[u8]>,
    header_start: u32,
    parent_id: VirtualNodeId,
    node_map: &VirtualNodeMap,
) -> EditorResult<VirtualNodeBody> {
    let section_index = IndexGroup::deserialize(reader)?;

    let mut models = Vec::with_capacity(section_index.entries.len() - 1);
    for entry in &section_index.entries[1..] {
        let name = section_index.get_entry_name(reader, entry)?;
        let data_start = section_index.get_entry_data_start(entry);

        reader.set_position(data_start as u64);

        let normals = Normals::deserialize(reader, header_start)?;

        let id = node_map.next_id();
        let node = VirtualNode {
            label: name,
            id,
            kind: VirtualNodeKind::Normals,
            parent: Some(parent_id),
            body: Deferred::evaluated(VirtualNodeBody {
                children: Vec::new(),
                inspectable: Some(Box::new(normals)),
            }),
        };
        node_map.insert(id, node);
        models.push(id);
    }

    Ok(VirtualNodeBody {
        children: models,
        inspectable: None,
    })
}

impl Normals {
    pub fn deserialize(reader: &mut RefCursor<[u8]>, header_start: u32) -> EditorResult<Self> {
        let _length = reader.read_u32::<BigEndian>()?;
        let _mdl0_offset = reader.read_i32::<BigEndian>()?;
        let data_offset = reader.read_i32::<BigEndian>()?;
        let _name_offset = reader.read_i32::<BigEndian>()?;
        let index = reader.read_u32::<BigEndian>()?;
        let component_count = reader.read_u32::<BigEndian>()?;
        let format = VectorFormat::deserialize(reader)?;
        let divisor = reader.read_u8()?;
        let stride = reader.read_u8()?;
        let normal_count = reader.read_u16::<BigEndian>()?;

        let normals_start = header_start as i64 + data_offset as i64;
        reader.set_position(normals_start as u64);

        let normals = match component_count {
            COMPONENTS_NORMAL => NormalData::Normal(deserialize_vector_data::<3>(
                reader,
                normal_count,
                format,
                divisor,
            )?),
            COMPONENTS_ALL => NormalData::All(deserialize_vector_data::<9>(
                reader,
                normal_count,
                format,
                divisor,
            )?),
            COMPONENTS_ANY => NormalData::Any(deserialize_vector_data::<3>(
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
            index,
            format,
            divisor,
            stride,
            normals,
        })
    }
}
