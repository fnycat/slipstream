use byteorder::{BigEndian, ReadBytesExt};

use crate::error::{CorruptionError, EditorResult};
use crate::format::brres::IndexGroup;
use crate::r#virtual::defer::Deferred;
use crate::r#virtual::node::{VirtualNode, VirtualNodeBody, VirtualNodeKind};
use crate::r#virtual::refs::{VirtualNodeId, VirtualRefCache};
use crate::{
    format::{
        encoding::{Deserialize, ReadArrayExt},
        mdl0::util::{ComponentFormat, deserialize_components},
    },
    shared::util::RefCursor,
};

const COMPONENTS_XY: u32 = 0x0;
const COMPONENTS_XYZ: u32 = 0x1;

#[derive(Debug, Clone, PartialEq)]
pub enum VertexData {
    XY(Vec<[f32; 2]>),
    XYZ(Vec<[f32; 3]>),
}

impl VertexData {
    pub const fn components(&self) -> usize {
        match self {
            Self::XY(_) => 2,
            Self::XYZ(_) => 3,
        }
    }

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

pub fn deserialize_virtual(
    reader: &mut RefCursor<[u8]>,
    header_start: u32,
    parent_id: VirtualNodeId,
    ref_cache: &VirtualRefCache,
) -> EditorResult<VirtualNodeBody> {
    let section_index = IndexGroup::deserialize(reader)?;

    let mut models = Vec::with_capacity(section_index.entries.len() - 1);
    for entry in &section_index.entries[1..] {
        let name = section_index.get_entry_name(reader, entry)?;

        let data_start = section_index.get_entry_data_start(entry);
        reader.set_position(data_start as u64);

        let model = Vertices::deserialize(reader, header_start)?;

        let id = ref_cache.next_id();
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

        ref_cache.insert(id, node);
        models.push(id);
    }

    Ok(VirtualNodeBody {
        children: models,
        inspectable: None,
    })
}

impl Vertices {
    pub fn deserialize(reader: &mut RefCursor<[u8]>, header_start: u32) -> EditorResult<Self> {
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

        // FIXME: This vertex data is not included in the lazy buffer.

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

        tracing::debug!("ended at {}", reader.position());

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
