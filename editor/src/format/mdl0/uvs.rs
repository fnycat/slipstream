use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    error::{CorruptionError, EditorResult},
    format::{
        brres::IndexGroup,
        encoding::{Deserialize, ReadArrayExt},
        mdl0::util::{VectorPrecision, deserialize_scalar_data, deserialize_vector_data},
    },
    node::{
        defer::Deferred,
        node::{VirtualNode, VirtualNodeBody, VirtualNodeKind},
        refs::{VirtualNodeId, VirtualNodeMap},
    },
    shared::util::RefCursor,
};

const COMPONENTS_S: u32 = 0x00;
const COMPONENTS_ST: u32 = 0x01;

#[derive(Debug, Clone, PartialEq)]
pub enum UvData {
    S(Vec<f32>),
    St(Vec<[f32; 2]>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Uvs {
    pub index: u32,
    pub format: VectorPrecision,
    pub stride: u8,
    pub uvs: UvData,
    pub bounding_volume_min: [f32; 2],
    pub bounding_volume_max: [f32; 2],
}

impl Uvs {
    pub fn deserialize(reader: &mut RefCursor<[u8]>, header_start: u32) -> EditorResult<Self> {
        let _length = reader.read_u32::<BigEndian>()?;
        let _mdl0_offset = reader.read_i32::<BigEndian>()?;
        let data_offset = reader.read_i32::<BigEndian>()?;
        let _name_offset = reader.read_i32::<BigEndian>()?;
        let index = reader.read_u32::<BigEndian>()?;
        let component_count = reader.read_u32::<BigEndian>()?;
        let format = VectorPrecision::deserialize(reader)?;
        let divisor = reader.read_u8()?;
        let stride = reader.read_u8()?;

        let uv_count = reader.read_u16::<BigEndian>()?;
        let bounding_volume_min = reader.read_f32_array::<2, BigEndian>()?;
        let bounding_volume_max = reader.read_f32_array::<2, BigEndian>()?;

        let uv_start = header_start as i64 + data_offset as i64;
        reader.set_position(uv_start as u64);

        let uvs = match component_count {
            COMPONENTS_S => UvData::S(deserialize_scalar_data(reader, uv_count, format, divisor)?),
            COMPONENTS_ST => UvData::St(deserialize_vector_data::<2>(
                reader, uv_count, format, divisor,
            )?),
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid UV format: {component_count} (expected 0, 1)"),
                    ..Default::default()
                }
                .into());
            }
        };

        Ok(Self {
            index,
            format,
            stride,
            uvs,
            bounding_volume_min,
            bounding_volume_max,
        })
    }
}

pub fn deserialize_virtual(
    reader: &mut RefCursor<[u8]>,
    header_start: u32,
    parent_id: VirtualNodeId,
    node_map: &VirtualNodeMap,
) -> EditorResult<VirtualNodeBody> {
    let section_index = IndexGroup::deserialize(reader)?;

    let mut children = Vec::with_capacity(section_index.entries.len() - 1);
    for entry in &section_index.entries[1..] {
        let name = section_index.get_entry_name(reader, entry)?;
        let data_start = section_index.get_entry_data_start(entry);

        reader.set_position(data_start as u64);

        let uvs = Uvs::deserialize(reader, header_start)?;

        let id = node_map.next_id();
        let node = VirtualNode {
            label: name,
            id,
            kind: VirtualNodeKind::Uvs,
            parent: Some(parent_id),
            body: Deferred::evaluated(VirtualNodeBody {
                children: Vec::new(),
                inspectable: Some(Box::new(uvs)),
            }),
        };

        node_map.insert(id, node);
        children.push(id);
    }

    Ok(VirtualNodeBody {
        children,
        inspectable: None,
    })
}
