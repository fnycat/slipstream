use bitfield_struct::bitenum;
use byteorder::{BigEndian, ReadBytesExt};

use crate::error::{CorruptionError, EditorError, EditorResult};
use crate::format::brres::IndexGroup;
use crate::format::mdl0::util::VectorDivisor;
use crate::node::defer::Deferred;
use crate::node::node::{VirtualNode, VirtualNodeBody, VirtualNodeKind};
use crate::node::refs::{VirtualNodeId, VirtualNodeMap};
use crate::{
    format::{
        encoding::Deserialize,
        mdl0::util::{VertexFormat, deserialize_vector_data},
    },
    shared::util::RefCursor,
};

const COMPONENTS_NORMAL: u32 = 0x0;
const COMPONENTS_ALL: u32 = 0x1;
const COMPONENTS_ANY: u32 = 0x2;

/// This enum has the same variant to value mapping as [`VertexFormat`] but leaves out
/// the formats that are invalid for normal data (i.e only signed formats).
///
/// [`VertexFormat`]: crate::format::mdl0::util::VertexFormat
#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum NormalFormat {
    Int8 = 1,
    Int16 = 3,
    Float32 = 4,
    /// Fallback value for `bitenum`, this variant should never be used.
    #[fallback]
    Invalid,
}

impl TryFrom<u32> for NormalFormat {
    type Error = EditorError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        // Literally the same as VertexFormat, except that it only supports signed
        // formats.
        Ok(match value {
            // 0 => Self::Uint8,
            1 => Self::Int8,
            // 2 => Self::Uint16,
            3 => Self::Int16,
            4 => Self::Float32,
            v => {
                return Err(CorruptionError {
                    reason: format!("invalid vertex format: {v} (expected 1, 3 or 4)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

impl Deserialize for NormalFormat {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

/// The type of normals that are stored in the normal buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NormalBufType {
    /// Only the normal itself is included in the buffer.
    NormalOnly,
    /// All three (normal/binormal/tangent) vectors are included in the buffer.
    All,
    /// Any one of the three vectors is in the buffer.
    Any,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NormalBufData {
    /// Only the normal.
    NormalOnly(Vec<[f32; 3]>),
    /// Includes all of the normal, bi-normal and tangent
    All(Vec<[f32; 9]>),
    /// Either the normal, bi-normal or tangent.
    Any(Vec<[f32; 3]>),
}

impl NormalBufData {
    pub fn ty(&self) -> NormalBufType {
        match self {
            Self::NormalOnly(_) => NormalBufType::NormalOnly,
            Self::All(_) => NormalBufType::All,
            Self::Any(_) => NormalBufType::Any,
        }
    }

    /// Returns the amount of entries in the buffer.
    ///
    /// This counts the [`All`] variant as one entry.
    ///
    /// [`All`]: NormalBufType::All
    pub fn len(&self) -> usize {
        match self {
            Self::NormalOnly(x) => x.len(),
            Self::All(x) => x.len(),
            Self::Any(x) => x.len(),
        }
    }
}

/// A large buffer of normals that the shape draw commands index into to draw their polygons.
///
/// The original file might store this data in a lower quality format, but the parser will always convert everything
/// to floats.
#[derive(Debug, Clone, PartialEq)]
pub struct NormalBuf {
    /// Index of this buffer into the `Normals` section of the model.
    pub index: u32,
    /// Scalar data type to use for the normal vectors.
    pub format: NormalFormat,
    /// The divisor is used to scale vectors at lower quality formats.
    ///
    /// For example if the vertex format is [`Int16`], then naively converting the
    /// vertices to floats would only give a range of -32,768 to 32,767 with whole integer intervals.
    ///
    /// The divisor is the power of 2 that is divided by the vertices to produce floats.
    /// I.e `float = int16 / 2^divisor`.
    ///
    /// [`Int16`]: NormalFormat::Int16
    pub divisor: u8,
    /// The size in bytes of each entry.
    pub stride: u8,
    /// The normal data.
    pub normals: NormalBufData,
}

impl NormalBuf {
    pub fn deserialize(reader: &mut RefCursor<[u8]>, header_start: u32) -> EditorResult<Self> {
        let _length = reader.read_u32::<BigEndian>()?;
        let _mdl0_offset = reader.read_i32::<BigEndian>()?;
        let data_offset = reader.read_i32::<BigEndian>()?;
        let _name_offset = reader.read_i32::<BigEndian>()?;
        let index = reader.read_u32::<BigEndian>()?;
        let component_count = reader.read_u32::<BigEndian>()?;
        let format = NormalFormat::deserialize(reader)?;
        let divisor = reader.read_u8()?;
        let stride = reader.read_u8()?;
        let normal_count = reader.read_u16::<BigEndian>()?;

        let normals_start = header_start as i64 + data_offset as i64;
        reader.set_position(normals_start as u64);

        let normals = match component_count {
            COMPONENTS_NORMAL => NormalBufData::NormalOnly(deserialize_vector_data::<3>(
                reader,
                normal_count as usize,
                VertexFormat::from(format),
                VectorDivisor::Custom(divisor),
            )?),
            COMPONENTS_ALL => NormalBufData::All(deserialize_vector_data::<9>(
                reader,
                normal_count as usize,
                VertexFormat::from(format),
                VectorDivisor::Custom(divisor),
            )?),
            COMPONENTS_ANY => NormalBufData::Any(deserialize_vector_data::<3>(
                reader,
                normal_count as usize,
                VertexFormat::from(format),
                VectorDivisor::Custom(divisor),
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

/// Deserializes all buffers in the `Normals` section of an MDL0 file.
#[tracing::instrument(skip_all, fields(parent_id))]
pub fn deserialize_normals_section(
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

        let normals = NormalBuf::deserialize(reader, header_start)?;

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
