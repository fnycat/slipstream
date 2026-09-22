use byteorder::ReadBytesExt;

use crate::{
    error::{CorruptionError, EditorError, EditorResult},
    format::{
        encoding::Deserialize,
        mdl0::gx::{
            GxOpCodeId::DrawQuads,
            call::CallDisplayList,
            draw::DrawOpCode,
            load_bp::LoadBpOpCode,
            load_cp::{LoadCpOpCode, MergedCpLoad},
            load_indexed::IndexedLoad,
            load_xf::LoadXfOpCode,
        },
    },
    shared::util::RefCursor,
};

pub mod call;
pub mod draw;
pub mod load_bp;
pub mod load_cp;
pub mod load_indexed;
pub mod load_xf;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GxOpCodeId {
    /// No-op
    Nop = 0x00,
    /// Load CP (command processor) register.
    LoadCp = 0x08,
    /// Load XF (transform unit) register.
    LoadXf = 0x10,
    /// Load indexed 3x3 position matrix.
    LoadIndexedPositionMatrix = 0x20,
    /// Load indexed 3x3 normal matrix.
    LoadIndexedNormalMatrix = 0x28,
    /// Load indexed texture matrix.
    LoadIndexedTextureMatrix = 0x30,
    /// Load indexed light object.
    LoadIndexedLightObject = 0x38,
    /// Call display list
    Call = 0x40,
    /// Unknown opcode.
    Unknown = 0x44,
    /// Invalidate vertex cache.
    InvalidateVertexCache = 0x48,
    /// Load BP (blitting processor) register.
    LoadBp = 0x61,
    /// Draw quads.
    DrawQuads = 0x80,
    /// Draw triangles.
    DrawTriangles = 0x90,
    /// Draw triangle strip.
    DrawTriangleStrip = 0x98,
    /// Draw triangle fan.
    DrawTriangleFan = 0xa0,
    /// Draw lines.
    DrawLines = 0xa8,
    /// Draw line strip.
    DrawLineStrip = 0xb0,
    /// Draw points.
    DrawPoints = 0xb8,
}

impl TryFrom<u8> for GxOpCodeId {
    type Error = EditorError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use GxOpCodeId::*;

        Ok(match value {
            0x00 => Nop,
            0x08 => LoadCp,
            0x10 => LoadXf,
            0x20 => LoadIndexedPositionMatrix,
            0x28 => LoadIndexedNormalMatrix,
            0x30 => LoadIndexedTextureMatrix,
            0x38 => LoadIndexedLightObject,
            0x40 => Call,
            0x44 => Unknown,
            0x48 => InvalidateVertexCache,
            0x61 => LoadBp,
            0x80 => DrawQuads,
            0x90 => DrawTriangles,
            0x98 => DrawTriangleStrip,
            0xa0 => DrawTriangleFan,
            0xa8 => DrawLines,
            0xb0 => DrawLineStrip,
            0xb8 => DrawPoints,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid GX opcode: {value:#04x}"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GxOpCode {
    Nop,
    LoadCp(LoadCpOpCode),
    LoadXf(LoadXfOpCode),
    LoadBp(LoadBpOpCode),
    LoadIndexedPosition(IndexedLoad),
    LoadIndexedNormal(IndexedLoad),
    LoadIndexedTextureMatrix(IndexedLoad),
    LoadIndexedLightObject(IndexedLoad),
    Call(CallDisplayList),
    InvalidateVertexCache,
    DrawQuads(DrawOpCode),
    DrawTriangles(DrawOpCode),
    DrawTriangleStrip(DrawOpCode),
    DrawTriangleFan(DrawOpCode),
    DrawLines(DrawOpCode),
    DrawLineStrip(DrawOpCode),
    DrawPoints(DrawOpCode),
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GxBytecode {
    commands: Vec<GxOpCode>,
}

impl GxBytecode {
    pub fn deserialize_vertex_declaration(
        reader: &mut RefCursor<[u8]>,
        section_end: u64,
    ) -> EditorResult<Self> {
        let mut commands = Vec::new();
        while reader.position() < section_end {
            let opcode = reader.read_u8()?;
            commands.push(match opcode {
                0x00 => continue,
                0x08 => GxOpCode::LoadCp(LoadCpOpCode::deserialize(reader)?),
                0x10 => GxOpCode::LoadXf(LoadXfOpCode::deserialize(reader)?),
                0x61 => GxOpCode::LoadBp(LoadBpOpCode::deserialize(reader)?),
                _ => {
                    return Err(CorruptionError {
                        reason: format!("invalid vertex declaration GX opcode: {opcode:#04x}"),
                        location: Some(reader.position()),
                    }
                    .into());
                }
            });
        }

        if reader.position() != section_end {
            return Err(CorruptionError {
                reason: format!("did not read correct amount of opcodes in vertex declaration GX bytecode ({} vs. {})", reader.position(), section_end),
                location: Some(reader.position())
            }.into());
        }

        Ok(Self { commands })
    }

    pub fn deserialize_vertex_data(
        reader: &mut RefCursor<[u8]>,
        vertex_decl: &Self,
        section_end: u64,
    ) -> EditorResult<Self> {
        let merged_cp_opcodes = MergedCpLoad::try_from(vertex_decl.commands.as_slice())?;

        let mut commands = Vec::new();
        while reader.position() < section_end {
            let opcode = reader.read_u8()?;
            commands.push(match opcode {
                0x00 => continue,
                0x20 => GxOpCode::LoadIndexedPosition(IndexedLoad::deserialize(reader)?),
                0x28 => GxOpCode::LoadIndexedNormal(IndexedLoad::deserialize(reader)?),
                0x30 => GxOpCode::LoadIndexedTextureMatrix(IndexedLoad::deserialize(reader)?),
                0x38 => GxOpCode::LoadIndexedLightObject(IndexedLoad::deserialize(reader)?),
                0x40 => GxOpCode::Call(CallDisplayList::deserialize(reader)?),
                0x44 => GxOpCode::Unknown,
                0x48 => GxOpCode::InvalidateVertexCache,
                0x80 => GxOpCode::DrawQuads(DrawOpCode::deserialize(reader, &merged_cp_opcodes)?),
                0x90 => {
                    GxOpCode::DrawTriangles(DrawOpCode::deserialize(reader, &merged_cp_opcodes)?)
                }
                0x98 => GxOpCode::DrawTriangleStrip(DrawOpCode::deserialize(
                    reader,
                    &merged_cp_opcodes,
                )?),
                0xa0 => {
                    GxOpCode::DrawTriangleFan(DrawOpCode::deserialize(reader, &merged_cp_opcodes)?)
                }
                0xa8 => GxOpCode::DrawLines(DrawOpCode::deserialize(reader, &merged_cp_opcodes)?),
                0xb0 => {
                    GxOpCode::DrawLineStrip(DrawOpCode::deserialize(reader, &merged_cp_opcodes)?)
                }
                0xb8 => GxOpCode::DrawPoints(DrawOpCode::deserialize(reader, &merged_cp_opcodes)?),
                _ => {
                    return Err(CorruptionError {
                        reason: format!("invalid vertex data GX opcode: {opcode:#04x}"),
                        location: Some(reader.position()),
                    }
                    .into());
                }
            });
        }

        if reader.position() != section_end {
            return Err(CorruptionError {
                reason: format!("did not read correct amount of opcodes in vertex declaration GX bytecode ({} vs. {})", reader.position(), section_end),
                location: Some(reader.position())
            }.into());
        }

        Ok(Self { commands })
    }
}
