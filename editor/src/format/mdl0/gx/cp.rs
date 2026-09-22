use bitfield_struct::{bitenum, bitfield};
use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    error::{CorruptionError, EditorResult},
    format::{encoding::Deserialize, mdl0::util::VectorFormat},
    shared::util::RefCursor,
};

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
    pub pos: VectorStorageMethod,
    #[bits(2)]
    pub norm: VectorStorageMethod,
    #[bits(2)]
    pub col0: VectorStorageMethod,
    #[bits(2)]
    pub col1: VectorStorageMethod,
    #[bits(15)]
    _padding: u16,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct CpSubCommand2 {
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
    _padding: u16,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct CpSubCommand3 {
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
pub struct CpSubCommand4 {
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
    #[bits(1)]
    _padding: bool,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct CpSubCommand5 {
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
