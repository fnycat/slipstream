use std::mem::MaybeUninit;

use bitfield_struct::{bitenum, bitfield};
use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    error::{CorruptionError, EditorError, EditorResult, InvalidInputError},
    format::{
        encoding::Deserialize,
        mdl0::{colors::ColorFormat, gx::GxOpCode, normals::NormalFormat, util::VertexFormat},
    },
    shared::util::RefCursor,
};

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum VectorStorage {
    #[fallback]
    NotPresent = 0b00,
    Direct = 0b01,
    Index8 = 0b10,
    Index16 = 0b11,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct CpSubCommand1 {
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
    pub pos_storage: VectorStorage,
    #[bits(2)]
    pub norm_storage: VectorStorage,
    #[bits(2)]
    pub col0_storage: VectorStorage,
    #[bits(2)]
    pub col1_storage: VectorStorage,
    #[bits(15)]
    _padding: u16,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct CpSubCommand2 {
    #[bits(2)]
    pub uv0_storage: VectorStorage,
    #[bits(2)]
    pub uv1_storage: VectorStorage,
    #[bits(2)]
    pub uv2_storage: VectorStorage,
    #[bits(2)]
    pub uv3_storage: VectorStorage,
    #[bits(2)]
    pub uv4_storage: VectorStorage,
    #[bits(2)]
    pub uv5_storage: VectorStorage,
    #[bits(2)]
    pub uv6_storage: VectorStorage,
    #[bits(2)]
    pub uv7_storage: VectorStorage,
    #[bits(16)]
    _padding: u16,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct CpSubCommand3 {
    pub pos_extended: bool,
    #[bits(3)]
    pub pos_format: VertexFormat,
    #[bits(5)]
    pub pos_divisor: u8,
    pub norm_extended: bool,
    #[bits(3)]
    pub norm_format: NormalFormat,
    pub col0_extended: bool,
    #[bits(3)]
    pub col0_format: ColorFormat,
    pub col1_extended: bool,
    #[bits(3)]
    pub col1_format: ColorFormat,
    pub uv0_extended: bool,
    #[bits(3)]
    pub uv0_format: VertexFormat,
    #[bits(5)]
    pub uv0_divisor: u8,
    pub dequant: bool,
    pub norm_l3: bool,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct CpSubCommand4 {
    pub uv1_extended: bool,
    #[bits(3)]
    pub uv1_format: VertexFormat,
    #[bits(5)]
    pub uv1_divisor: u8,
    pub uv2_extended: bool,
    #[bits(3)]
    pub uv2_format: VertexFormat,
    #[bits(5)]
    pub uv2_divisor: u8,
    pub uv3_extended: bool,
    #[bits(3)]
    pub uv3_format: VertexFormat,
    #[bits(5)]
    pub uv3_divisor: u8,
    pub uv4_extended: bool,
    #[bits(3)]
    pub uv4_format: VertexFormat,
    #[bits(1)]
    _padding: bool,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct CpSubCommand5 {
    #[bits(5)]
    pub uv4_divisor: u8,
    pub uv5_extended: bool,
    #[bits(3)]
    pub uv5_format: VertexFormat,
    #[bits(5)]
    pub uv5_divisor: u8,
    pub uv6_extended: bool,
    #[bits(3)]
    pub uv6_format: VertexFormat,
    #[bits(5)]
    pub uv6_divisor: u8,
    pub uv7_extended: bool,
    #[bits(3)]
    pub uv7_format: VertexFormat,
    #[bits(5)]
    pub uv7_divisor: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadCpOpCode {
    C1(CpSubCommand1),
    C2(CpSubCommand2),
    C3(CpSubCommand3),
    C4(CpSubCommand4),
    C5(CpSubCommand5),
}

impl Deserialize for LoadCpOpCode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let byte = reader.read_u8()?;
        let word = reader.read_u32::<BigEndian>()?;

        Ok(match byte {
            0x50 => Self::C1(CpSubCommand1::from_bits(word)),
            0x60 => Self::C2(CpSubCommand2::from_bits(word)),
            0x70 => Self::C3(CpSubCommand3::from_bits(word)),
            0x80 => Self::C4(CpSubCommand4::from_bits(word)),
            0x90 => Self::C5(CpSubCommand5::from_bits(word)),
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid LoadCP subcommand: {byte:#04x}"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}
