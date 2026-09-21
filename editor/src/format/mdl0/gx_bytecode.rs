use bitfield_struct::{bitenum, bitfield};
use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    error::{CorruptionError, EditorError, EditorResult},
    format::{
        encoding::{Deserialize, ReadArrayExt},
        mdl0::util::VectorFormat,
    },
    shared::util::RefCursor,
};

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

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum VectorStorageMethod {
    #[fallback]
    NotPresent = 0b00,
    Direct = 0b01,
    Index8 = 0b10,
    Index16 = 0b11,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct C1 {
    pub pm: bool,
    pub tm0: bool,
    pub tm1: bool,
    pub tm2: bool,
    pub tm3: bool,
    pub tm4: bool,
    pub tm5: bool,
    pub tm6: bool,
    pub tm7: bool,
    #[bits(2)]
    pub pos: VectorStorageMethod,
    #[bits(2)]
    pub norm: VectorStorageMethod,
    #[bits(2)]
    pub col0: VectorStorageMethod,
    #[bits(2)]
    pub col1: VectorStorageMethod,
    #[bits(15)]
    pub _unused: u16,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct C2 {
    #[bits(2)]
    pub tex0: VectorStorageMethod,
    #[bits(2)]
    pub tex1: VectorStorageMethod,
    #[bits(2)]
    pub tex2: VectorStorageMethod,
    #[bits(2)]
    pub tex3: VectorStorageMethod,
    #[bits(2)]
    pub tex4: VectorStorageMethod,
    #[bits(2)]
    pub tex5: VectorStorageMethod,
    #[bits(2)]
    pub tex6: VectorStorageMethod,
    #[bits(2)]
    pub tex7: VectorStorageMethod,
    #[bits(16)]
    pub _unused: u16,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct C3 {
    pub pos_e: bool,
    #[bits(3)]
    pub pos_format: VectorFormat,
    #[bits(5)]
    pub pos_divisor: u8,
    pub norm_e: bool,
    #[bits(3)]
    pub norm_format: VectorFormat,
    pub col0_e: bool,
    #[bits(3)]
    pub col0_format: VectorFormat,
    pub col1_e: bool,
    #[bits(3)]
    pub col1_format: VectorFormat,
    pub tex0_e: bool,
    #[bits(3)]
    pub tex0_format: VectorFormat,
    #[bits(5)]
    pub tex0_divisor: u8,
    pub dequant: bool,
    pub norm_l3: bool,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct C4 {
    pub tex1_e: bool,
    #[bits(3)]
    pub tex1_format: VectorFormat,
    #[bits(5)]
    pub tex1_divisor: u8,
    pub tex2_e: bool,
    #[bits(3)]
    pub tex2_format: VectorFormat,
    #[bits(5)]
    pub tex2_divisor: u8,
    pub tex3_e: bool,
    #[bits(3)]
    pub tex3_format: VectorFormat,
    #[bits(5)]
    pub tex3_divisor: u8,
    pub tex4_e: bool,
    #[bits(3)]
    pub tex4_format: VectorFormat,
    #[bits(1, default = false)]
    pub _unused: bool,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct C5 {
    #[bits(5)]
    pub tex4_divisor: u8,
    pub tex5_e: bool,
    #[bits(3)]
    pub tex5_format: VectorFormat,
    #[bits(5)]
    pub tex5_divisor: u8,
    pub tex6_e: bool,
    #[bits(3)]
    pub tex6_format: VectorFormat,
    #[bits(5)]
    pub tex6_divisor: u8,
    pub tex7_e: bool,
    #[bits(3)]
    pub tex7_format: VectorFormat,
    #[bits(5)]
    pub tex7_divisor: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadCpSubCommand {
    C1(C1),
    C2(C2),
    C3(C3),
    C4(C4),
    C5(C5),
}

impl Deserialize for LoadCpSubCommand {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let byte = reader.read_u8()?;
        let word = reader.read_u32::<BigEndian>()?;

        Ok(match byte {
            0x50 => Self::C1(C1::from_bits(word)),
            0x60 => Self::C2(C2::from_bits(word)),
            0x70 => Self::C3(C3::from_bits(word)),
            0x80 => Self::C4(C4::from_bits(word)),
            0x90 => Self::C5(C5::from_bits(word)),
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid LoadCP subcommand: {byte}"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

pub struct CpOpCode {
    pub subcommand: LoadCpSubCommand,
    pub params: [u8; 4],
}

impl Deserialize for CpOpCode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let subcommand = LoadCpSubCommand::deserialize(reader)?;
        let params = reader.read_u8_array::<4>()?;

        Ok(Self { subcommand, params })
    }
}

pub enum GxOpCode {
    Nop,
    LoadCp(CpOpCode),
}

#[derive(Debug, Clone)]
pub struct GxBytecode {}

impl Deserialize for GxBytecode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        todo!()
    }
}
