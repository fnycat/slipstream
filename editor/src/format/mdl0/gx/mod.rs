use byteorder::ReadBytesExt;

use crate::{
    error::EditorResult,
    format::{
        encoding::Deserialize,
        mdl0::gx::{bp::LoadBpOpCode, cp::LoadCpOpCode, xf::LoadXfOpCode},
    },
    shared::util::RefCursor,
};

pub mod bp;
pub mod cp;
pub mod xf;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GxOpCodeId {
    /// No-op
    Nop = 0x00,
    /// Load CP (command processor) register.
    LoadCp = 0x08,
    /// Load XF (transform unit) register.
    LoadXf = 0x10,
    /// Load indexed 3x3 position matrix.
    LoadPosMat = 0x20,
    /// Load indexed 3x3 normal matrix.
    LoadNorMat = 0x28,
    /// Load indexed texture matrix.
    LoadTexMat = 0x30,
    /// Load indexed light object.
    LoadLightObj = 0x38,
    /// Call display list
    Call = 0x40,
    /// Unknown opcode.
    Unknown = 0x44,
    /// Invalidate vertex cache.
    InvalidateVCache = 0x48,
    /// Load BP (blitting processor) register.
    LoadBp = 0x61,
    /// Draw quads.
    DrawQuads = 0x80,
    /// Draw triangles.
    DrawTris = 0x90,
    /// Draw triangle strip.
    DrawTriStrip = 0x98,
    /// Draw triangle fan.
    DrawTriFan = 0xa0,
    /// Draw lines.
    DrawLines = 0xa8,
    /// Draw line strip.
    DrawLineStrip = 0xb0,
    /// Draw points.
    DrawPoints = 0xb8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GxOpCode {
    Nop,
    LoadCp(LoadCpOpCode),
    LoadXf(LoadXfOpCode),
    LoadBp(LoadBpOpCode),
}

#[derive(Debug, Clone)]
pub struct GxBytecode {
    commands: Vec<GxOpCode>,
}

impl GxBytecode {
    pub fn deserialize(reader: &mut RefCursor<[u8]>, section_end: u64) -> EditorResult<Self> {
        let mut commands = Vec::new();
        while reader.position() < section_end {
            let opcode = reader.read_u8()?;
            commands.push(match opcode {
                0x00 => continue,
                0x08 => GxOpCode::LoadCp(LoadCpOpCode::deserialize(reader)?),
                0x10 => GxOpCode::LoadXf(LoadXfOpCode::deserialize(reader)?),
                0x61 => GxOpCode::LoadBp(LoadBpOpCode::deserialize(reader)?),
                _ => todo!("{opcode:#04x}"),
            });
        }

        Ok(Self { commands })
    }
}
