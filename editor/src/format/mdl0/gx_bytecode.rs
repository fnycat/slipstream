use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    error::{CorruptionError, EditorError, EditorResult},
    format::encoding::{Deserialize, ReadArrayExt},
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

#[derive(Debug, Clone, PartialEq)]
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
    pub pos: C1BitFlag,
    pub norm: C1BitFlag,
    pub col0: C1BitFlag,
    pub col1: C1BitFlag,
}

impl Deserialize for C1 {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;

        let pm = (word & 0x01) == 0x01;
        let tm0 = ((word << 1) & 0x01) == 0x01;
        let tm1 = ((word << 2) & 0x01) == 0x01;
        let tm2 = ((word << 3) & 0x01) == 0x01;
        let tm3 = ((word << 4) & 0x01) == 0x01;
        let tm4 = ((word << 5) & 0x01) == 0x01;
        let tm5 = ((word << 6) & 0x01) == 0x01;
        let tm6 = ((word << 7) & 0x01) == 0x01;
        let tm7 = ((word << 8) & 0x01) == 0x01;

        let pos = C1BitFlag::try_from(((word << 9) & 0x03) as u8)?;
        let norm = C1BitFlag::try_from(((word << 11) & 0x03) as u8)?;
        let col0 = C1BitFlag::try_from(((word << 13) & 0x03) as u8)?;
        let col1 = C1BitFlag::try_from(((word << 15) & 0x03) as u8)?;

        Ok(Self {
            pm,
            tm0,
            tm1,
            tm2,
            tm3,
            tm4,
            tm5,
            tm6,
            tm7,
            pos,
            norm,
            col0,
            col1,
        })
    }
}

macro_rules! impl_c1 {
    ($($id:expr),*) => {
        paste::paste! {
            #[derive(Debug, Clone, PartialEq)]
            pub struct C1 {
                pub pm: bool,
                $(pub [< tm $id >]: bool),*
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

                    })
                }
            }
        }
    }
}

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
pub enum LoadCpSubCommand {
    C1(C1),
    C2(C2),
    C3(C3),
}

impl Deserialize for LoadCpSubCommand {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let byte = reader.read_u8()?;
        Ok(match byte {
            0x50 => Self::C1(C1::deserialize(reader)?),
            0x60 => Self::C2(C2::deserialize(reader)?),
            0x70 => Self::C3(C3::deserialize(reader)?),
            _ => todo!(),
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
