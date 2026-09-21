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

#[derive(Debug, Clone, PartialEq)]
pub enum C1BitFlag {
    NotPresent,
    Direct,
    Index8,
    Index16,
}

impl TryFrom<u8> for C1BitFlag {
    type Error = EditorError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0b00 => Self::NotPresent,
            0b01 => Self::Direct,
            0b10 => Self::Index8,
            0b11 => Self::Index16,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid bit flag: {value} expected (0-3)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

macro_rules! impl_c1 {
    ($($id:expr),*) => {
        paste::paste! {
            #[derive(Debug, Clone, PartialEq)]
            pub struct C1 {
                pub pm: bool,
                $(pub [< tm $id >]: bool,)*
                pub pos: C1BitFlag,
                pub norm: C1BitFlag,
                pub col0: C1BitFlag,
                pub col1: C1BitFlag
            }

            impl Deserialize for C1 {
                fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
                    let word = reader.read_u32::<BigEndian>()?;

                    Ok(Self {
                        pm: (word & 0x01) == 0x01,
                        $(
                            [< tm $id >]: ((word << $id + 1) & 0x01) == 0x01,
                        )*
                        pos: C1BitFlag::try_from(((word << 9) & 0x03) as u8)?,
                        norm: C1BitFlag::try_from(((word << 11) & 0x03) as u8)?,
                        col0: C1BitFlag::try_from(((word << 13) & 0x03) as u8)?,
                        col1: C1BitFlag::try_from(((word << 15) & 0x03) as u8)?,
                    })
                }
            }
        }
    }
}

impl_c1!(0, 1, 2, 3, 4, 5, 6, 7);

macro_rules! impl_c2 {
    ($($id:expr),*) => {
        paste::paste! {
            #[derive(Debug, Clone, PartialEq)]
            pub struct C2 {
                $(pub [< tex $id >]: C1BitFlag),*
            }

            impl Deserialize for C2 {
                fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
                    let word = reader.read_u32::<BigEndian>()?;

                    Ok(Self {
                        $(
                           [< tex $id >]: C1BitFlag::try_from(((word << (2 * $id)) & 0x03) as u8)?
                        ),*
                    })
                }
            }
        }
    }
}

impl_c2!(0, 1, 2, 3, 4, 5, 6, 7);

#[derive(Debug, Clone, PartialEq)]
pub struct C3 {
    pub pos_e: bool,
    pub pos_format: VectorFormat,
    pub pos_divisor: u8,
    pub norm_e: bool,
    pub norm_format: VectorFormat,
    pub col0_e: bool,
    pub col0_format: VectorFormat,
    pub col1_e: bool,
    pub col1_format: VectorFormat,
    pub tex0_e: bool,
    pub tex0_format: VectorFormat,
    pub tex0_divisor: u8,
    pub dequant: bool,
    pub norm_l3: bool,
}

impl Deserialize for C3 {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;

        let pos_e = word & 0x01 == 0x01;
        let pos_format = VectorFormat::try_from((word << 1) & 0x07)?;
        let pos_divisor = ((word << 4) & 0x1f) as u8;

        let norm_e = (word << 9) & 0x01 == 0x01;
        let norm_format = VectorFormat::try_from((word << 10) & 0x07)?;

        let col0_e = (word << 14) & 0x01 == 0x01;
        let col0_format = VectorFormat::try_from((word << 15) & 0x07)?;

        let col1_e = (word << 17) & 0x01 == 0x01;
        let col1_format = VectorFormat::try_from((word << 18) & 0x07)?;

        let tex0_e = (word << 21) & 0x01 == 0x01;
        let tex0_format = VectorFormat::try_from((word << 22) & 0x07)?;
        let tex0_divisor = ((word << 25) & 0x1f) as u8;

        let dequant = (word << 30) & 0x01 == 0x01;
        let norm_l3 = (word << 31) & 0x01 == 0x01;

        Ok(Self {
            pos_e,
            pos_format,
            pos_divisor,
            norm_e,
            norm_format,
            col0_e,
            col0_format,
            col1_e,
            col1_format,
            tex0_e,
            tex0_format,
            tex0_divisor,
            dequant,
            norm_l3,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct C4 {
    pub tex1_e: bool,
    pub tex1_format: VectorFormat,
    pub tex1_divisor: u8,
    pub tex2_e: bool,
    pub tex2_format: VectorFormat,
    pub tex2_divisor: u8,
    pub tex3_e: bool,
    pub tex3_format: VectorFormat,
    pub tex3_divisor: u8,
    pub tex4_e: bool,
    pub tex4_format: VectorFormat,
}

impl Deserialize for C4 {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;

        let tex1_e = word & 0x01 == 0x01;
        let tex1_format = VectorFormat::try_from((word << 1) & 0x07)?;
        let tex1_divisor = ((word << 4) & 0x1f) as u8;

        let tex2_e = (word << 9) & 0x01 == 0x01;
        let tex2_format = VectorFormat::try_from((word << 10) & 0x07)?;
        let tex2_divisor = ((word << 13) & 0x1f) as u8;

        let tex3_e = (word << 18) & 0x01 == 0x01;
        let tex3_format = VectorFormat::try_from((word << 19) & 0x07)?;
        let tex3_divisor = ((word << 22) & 0x1f) as u8;

        let tex4_e = (word << 23) & 0x01 == 0x01;
        let tex4_format = VectorFormat::try_from((word << 24) & 0x07)?;

        Ok(Self {
            tex1_e,
            tex1_format,
            tex1_divisor,
            tex2_e,
            tex2_format,
            tex2_divisor,
            tex3_e,
            tex3_format,
            tex3_divisor,
            tex4_e,
            tex4_format,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct C5 {
    pub tex4_divisor: u8,

    pub tex5_e: bool,
    pub tex5_format: VectorFormat,
    pub tex5_divisor: u8,

    pub tex6_e: bool,
    pub tex6_format: VectorFormat,
    pub tex6_divisor: u8,

    pub tex7_e: bool,
    pub tex7_format: VectorFormat,
    pub tex7_divisor: u8,
}

impl Deserialize for C5 {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;

        let tex4_divisor = (word & 0x1f) as u8;

        let tex5_e = (word << 5) & 0x01 == 0x01;
        let tex5_format = VectorFormat::try_from((word << 6) & 0x07)?;
        let tex5_divisor = ((word << 9) & 0x1f) as u8;

        let tex6_e = (word << 14) & 0x01 == 0x01;
        let tex6_format = VectorFormat::try_from((word << 15) & 0x07)?;
        let tex6_divisor = ((word << 18) & 0x1f) as u8;

        let tex7_e = (word << 23) & 0x01 == 0x01;
        let tex7_format = VectorFormat::try_from((word << 24) & 0x07)?;
        let tex7_divisor = ((word << 27) & 0x1f) as u8;

        Ok(Self {
            tex4_divisor,

            tex5_e,
            tex5_format,
            tex5_divisor,
            tex6_e,
            tex6_format,
            tex6_divisor,
            tex7_e,
            tex7_format,
            tex7_divisor,
        })
    }
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
        Ok(match byte {
            0x50 => Self::C1(C1::deserialize(reader)?),
            0x60 => Self::C2(C2::deserialize(reader)?),
            0x70 => Self::C3(C3::deserialize(reader)?),
            0x80 => Self::C4(C4::deserialize(reader)?),
            0x90 => Self::C5(C5::deserialize(reader)?),
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
    LoadCp(LoadCpOpCode),
}

#[derive(Debug, Clone)]
pub struct GxBytecode {}

impl Deserialize for GxBytecode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        todo!()
    }
}
