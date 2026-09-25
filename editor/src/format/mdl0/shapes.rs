use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    error::{CorruptionError, EditorError, EditorResult, InvalidInputError},
    format::{
        brres::IndexGroup,
        encoding::{Deserialize, ReadArrayExt, ReadStringExt},
        mdl0::gx::{
            GxBytecode, GxOpCode,
            load_cp::{CpVatA, CpVatB, CpVatC, CpVcdHi, CpVcdLo, LoadCpOpCode},
            load_xf::{LoadXfOpCode, LoadXfPayload},
        },
    },
    node::{
        defer::Deferred,
        node::{VirtualNode, VirtualNodeBody, VirtualNodeKind},
        refs::{VirtualNodeId, VirtualNodeMap},
    },
    shared::util::RefCursor,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoneTableEntry {
    pub bone_id1: i16,
    pub bone_id2: i16,
}

impl Deserialize for BoneTableEntry {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let bone_id1 = reader.read_i16::<BigEndian>()?;
        let bone_id2 = reader.read_i16::<BigEndian>()?;

        Ok(Self { bone_id1, bone_id2 })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoneTable {
    pub entries: Vec<BoneTableEntry>,
}

impl Deserialize for BoneTable {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let entry_count = reader.read_u32::<BigEndian>()?;

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            entries.push(BoneTableEntry::deserialize(reader)?);
        }

        Ok(Self { entries })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShapeModifier {
    None,
    ChangeCurrentMatrix,
    Invisible,
}

impl TryFrom<u32> for ShapeModifier {
    type Error = EditorError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::None,
            1 => Self::ChangeCurrentMatrix,
            2 => Self::Invisible,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid object modifier: {value} (expected 0, 1 or 2)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

impl Deserialize for ShapeModifier {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoneBind {
    Single(u32),
    Table(BoneTable),
}

/// Setup bytecode for a shape.
///
/// This describes the formats of all the buffers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GxVertexDeclaration {
    /// The low vertex control descriptor.
    pub vcd_lo: CpVcdLo,
    /// The high vertex control descriptor.
    pub vcd_hi: CpVcdHi,
    /// The first vertex attribute table.
    pub vat_a: CpVatA,
    /// The second vertex attribute table.
    pub vat_b: CpVatB,
    /// The third vertex attribute table.
    pub vat_c: CpVatC,
    pub xf: Vec<LoadXfPayload>,
}

impl TryFrom<GxBytecode> for GxVertexDeclaration {
    type Error = EditorError;

    fn try_from(value: GxBytecode) -> Result<Self, Self::Error> {
        let mut vcd_lo = None;
        let mut vcd_hi = None;
        let mut vat_a = None;
        let mut vat_b = None;
        let mut vat_c = None;
        let mut xf = None;

        for opcode in value.commands {
            match opcode {
                GxOpCode::LoadXf(LoadXfOpCode { loads }) => xf = Some(loads),
                GxOpCode::LoadCp(LoadCpOpCode::VcdLo(x)) => vcd_lo = Some(x),
                GxOpCode::LoadCp(LoadCpOpCode::VcdHi(x)) => vcd_hi = Some(x),
                GxOpCode::LoadCp(LoadCpOpCode::VatA(x)) => vat_a = Some(x),
                GxOpCode::LoadCp(LoadCpOpCode::VatB(x)) => vat_b = Some(x),
                GxOpCode::LoadCp(LoadCpOpCode::VatC(x)) => vat_c = Some(x),
                _ => {}
            }
        }

        Ok(Self {
            xf: xf.ok_or_else(|| {
                EditorError::from(InvalidInputError {
                    reason: String::from("vertex declaration did not contain LoadXF opcode"),
                    ..Default::default()
                })
            })?,
            vcd_lo: vcd_lo.ok_or_else(|| {
                EditorError::from(InvalidInputError {
                    reason: String::from("vertex declaration did not contain cp1 opcode"),
                    ..Default::default()
                })
            })?,
            vcd_hi: vcd_hi.ok_or_else(|| {
                EditorError::from(InvalidInputError {
                    reason: String::from("vertex declaration did not contain cp2 opcode"),
                    ..Default::default()
                })
            })?,
            vat_a: vat_a.ok_or_else(|| {
                EditorError::from(InvalidInputError {
                    reason: String::from("vertex declaration did not contain cp3 opcode"),
                    ..Default::default()
                })
            })?,
            vat_b: vat_b.ok_or_else(|| {
                EditorError::from(InvalidInputError {
                    reason: String::from("vertex declaration did not contain cp4 opcode"),
                    ..Default::default()
                })
            })?,
            vat_c: vat_c.ok_or_else(|| {
                EditorError::from(InvalidInputError {
                    reason: String::from("vertex declaration did not contain cp5 opcode"),
                    ..Default::default()
                })
            })?,
        })
    }
}

/// A shape/object/polygon describes how the model should be rendered.
///
/// It contains the actual draw commands for the Broadway GPU to execute.
#[derive(Debug, Clone)]
pub struct Shape {
    pub array_flags: u32,
    pub modifier: ShapeModifier,
    /// This shape's index in the `Shapes` section.
    pub index: u32,
    pub vertex_count: u32,
    pub face_count: u32,
    /// The vertex buffer to use for indexed draws.
    ///
    /// This is an index into the `Vertices` section of the model.
    pub vertex_array_id: u16,
    /// The normal buffer to use for indexed draws.
    ///
    /// This is an index into the `Normals` section of the model.
    pub normal_array_id: u16,
    /// The color buffer to use for indexed draws.
    ///
    /// This is an index into the `Colors` section of the model.
    pub color_array_ids: [u16; 2],
    /// The UV buffer to use for indexed draws.
    ///
    /// This is an index into the `UVs` section of the model.
    pub uv_array_ids: [u16; 8],

    pub bone_bind: BoneBind,
    /// Setup bytecode for the buffer formats.
    ///
    /// See [`GxVertexDeclaration`] for more info.
    pub vertex_decl: GxVertexDeclaration,
    /// Bytecode containing this shape's draw calls.
    pub vertex_data_gx: GxBytecode,
}

impl Deserialize for Shape {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let object_start = reader.position();
        let length = reader.read_u32::<BigEndian>()?;
        let mdl0_offset = reader.read_i32::<BigEndian>()?;
        let bone_index = match reader.read_i32::<BigEndian>()? {
            -1 => None,
            v => Some(v as u32),
        };

        let cp_vtx = reader.read_u32::<BigEndian>()?;
        let cp_tex = reader.read_u32::<BigEndian>()?;
        let xf_nor_spec = reader.read_u32::<BigEndian>()?;

        let definitions_buffer_size = reader.read_u32::<BigEndian>()?;
        let definitions_size = reader.read_u32::<BigEndian>()?;
        let definitions_offset = reader.read_i32::<BigEndian>()?;
        let vertex_buffer_size = reader.read_u32::<BigEndian>()?;
        let vertex_data_size = reader.read_u32::<BigEndian>()?;
        let vertex_data_offset = reader.read_i32::<BigEndian>()?;
        let array_flags = reader.read_u32::<BigEndian>()?;
        let modifier = ShapeModifier::deserialize(reader)?;
        let _name_offset = reader.read_u32::<BigEndian>()?;
        let index = reader.read_u32::<BigEndian>()?;
        let vertex_count = reader.read_u32::<BigEndian>()?;
        let face_count = reader.read_u32::<BigEndian>()?;
        let vertex_array_id = reader.read_u16::<BigEndian>()?;
        let normal_array_id = reader.read_u16::<BigEndian>()?;
        let color_array_ids = reader.read_u16_array::<2, BigEndian>()?;
        let uv_array_ids = reader.read_u16_array::<8, BigEndian>()?;
        let _unknown = reader.read_u32::<BigEndian>()?;
        let bone_table_offset = reader.read_u32::<BigEndian>()?;

        reader.set_position(object_start + bone_table_offset as u64);

        let bone_bind = match bone_index {
            Some(index) => BoneBind::Single(index),
            None => BoneBind::Table(BoneTable::deserialize(reader)?),
        };

        // The definitions and vertices offsets are relative to their fields, not the the file start.
        const VERTEX_DECL_INTERNAL_OFFSET: u64 = 0x20;
        const VERTEX_DATA_INTERNAL_OFFSET: u64 = 0x24;

        let definitions_start =
            object_start as i64 + VERTEX_DECL_INTERNAL_OFFSET as i64 + definitions_offset as i64;

        let definitions_end = definitions_start + definitions_size as i64;

        reader.set_position(definitions_start as u64);

        tracing::trace!("Reading vertex declaration GX bytecode");
        let vertex_decl = GxVertexDeclaration::try_from(
            GxBytecode::deserialize_vertex_declaration(reader, definitions_end as u64)?,
        )?;

        let vertices_start =
            object_start as i64 + VERTEX_DATA_INTERNAL_OFFSET as i64 + vertex_data_offset as i64;

        let vertices_end = vertices_start + vertex_data_size as i64;

        reader.set_position(vertices_start as u64);

        tracing::trace!("Reading vertex data GX bytecode");
        let vertex_data_gx =
            GxBytecode::deserialize_vertex_data(reader, &vertex_decl, vertices_end as u64)?;

        Ok(Self {
            vertex_count,
            face_count,
            vertex_array_id,
            normal_array_id,
            color_array_ids,
            uv_array_ids,

            array_flags,
            modifier,
            index,

            bone_bind,
            vertex_decl,
            vertex_data_gx,
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

    let mut children = Vec::with_capacity(section_index.entries.len());
    for entry in &section_index.entries[1..] {
        let name = section_index.get_entry_name(reader, entry)?;
        let data_start = section_index.get_entry_data_start(entry);

        reader.set_position(data_start as u64);

        let object = Shape::deserialize(reader)?;

        let id = node_map.next_id();
        let node = VirtualNode {
            label: name,
            id,
            kind: VirtualNodeKind::Shape,
            parent: Some(parent_id),
            body: Deferred::evaluated(VirtualNodeBody {
                children: Vec::new(),
                inspectable: Some(Box::new(object)),
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
