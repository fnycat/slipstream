use std::borrow::Cow;

use byteorder::{BigEndian, ReadBytesExt};

use crate::error::{CorruptionError, EditorResult};
use crate::format::brres::IndexGroup;
use crate::format::mdl0::util::VectorDivisor;
use crate::node::defer::Deferred;
use crate::node::node::{VirtualNode, VirtualNodeBody, VirtualNodeKind};
use crate::node::refs::{VirtualNodeId, VirtualNodeMap};
use crate::panes::viewer::translator::TranslatedModel;
use crate::{
    format::{
        encoding::{Deserialize, ReadArrayExt},
        mdl0::util::{VertexFormat, deserialize_vector_data},
    },
    shared::util::RefCursor,
};

const COMPONENTS_XY: u32 = 0x0;
const COMPONENTS_XYZ: u32 = 0x1;

#[derive(Debug, Clone, PartialEq)]
pub enum VertexPositionType {
    Xy,
    Xyz,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VertexBufData {
    Xy(Vec<[f32; 2]>),
    Xyz(Vec<[f32; 3]>),
}

impl VertexBufData {
    pub const fn ty(&self) -> VertexPositionType {
        match self {
            Self::Xy(_) => VertexPositionType::Xy,
            Self::Xyz(_) => VertexPositionType::Xyz,
        }
    }

    pub const fn components(&self) -> usize {
        match self {
            Self::Xy(_) => 2,
            Self::Xyz(_) => 3,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Xy(verts) => bytemuck::cast_slice(verts),
            Self::Xyz(verts) => bytemuck::cast_slice(verts),
        }
    }

    /// Returns the buffer size in bytes.
    pub fn size(&self) -> usize {
        match self {
            Self::Xy(verts) => 2 * size_of::<f32>() * verts.len(),
            Self::Xyz(verts) => 3 * size_of::<f32>() * verts.len(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Xy(verts) => verts.len(),
            Self::Xyz(verts) => verts.len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VertexBuf {
    pub index: u32,
    pub format: VertexFormat,
    pub divisor: u8,
    pub stride: u8,
    pub bounding_volume_min: [f32; 3],
    pub bounding_volume_max: [f32; 3],
    pub vertices: VertexBufData,
}

impl VertexBuf {
    pub fn deserialize(reader: &mut RefCursor<[u8]>, header_start: u32) -> EditorResult<Self> {
        let _length = reader.read_u32::<BigEndian>()?;
        let _mdl0_offset = reader.read_i32::<BigEndian>()?;
        let data_offset = reader.read_i32::<BigEndian>()?;
        let _name_offset = reader.read_i32::<BigEndian>()?;
        let index = reader.read_u32::<BigEndian>()?;
        let component_count = reader.read_u32::<BigEndian>()?;
        let format = VertexFormat::deserialize(reader)?;
        let divisor = reader.read_u8()?;
        let stride = reader.read_u8()?;
        let vertex_count = reader.read_u16::<BigEndian>()?;
        let bounding_volume_min = reader.read_f32_array::<3, BigEndian>()?;
        let bounding_volume_max = reader.read_f32_array::<3, BigEndian>()?;

        tracing::trace!("Reading {vertex_count} vertices");

        let vertices_start = header_start as i64 + data_offset as i64;
        reader.set_position(vertices_start as u64);

        let vertices = match component_count {
            COMPONENTS_XY => VertexBufData::Xy(deserialize_vector_data::<2>(
                reader,
                vertex_count as usize,
                format,
                VectorDivisor::Custom(divisor),
            )?),
            COMPONENTS_XYZ => VertexBufData::Xyz(deserialize_vector_data::<3>(
                reader,
                vertex_count as usize,
                format,
                VectorDivisor::Custom(divisor),
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

        tracing::debug!("ended at {}", reader.position());

        Ok(Self {
            index,
            vertices,
            format,
            divisor,
            stride,
            bounding_volume_min,
            bounding_volume_max,
        })
    }
}

#[tracing::instrument(skip_all, fields(parent_id))]
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

        let model = VertexBuf::deserialize(reader, header_start)?;

        let id = node_map.next_id();
        let node = VirtualNode {
            label: name,
            id,
            kind: VirtualNodeKind::Vertices,
            parent: Some(parent_id),
            body: Deferred::evaluated(VirtualNodeBody {
                children: Vec::new(),
                inspectable: Some(Box::new(model)),
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
