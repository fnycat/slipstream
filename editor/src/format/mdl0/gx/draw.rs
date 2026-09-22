use byteorder::{BigEndian, ReadBytesExt};
use glam::Vec4Swizzles;

use crate::{
    error::EditorResult,
    format::mdl0::{
        colors::deserialize_color,
        gx::load_cp::{LoadCpOpCode, MergedCpLoad, VectorStorage},
        util::{VectorDivisor, VertexFormat, deserialize_scalar, deserialize_vector},
    },
    shared::util::RefCursor,
};

#[derive(Debug, Clone, PartialEq)]
pub enum DirectPosition {
    Xy([f32; 2]),
    Xyz([f32; 3]),
}

impl DirectPosition {
    pub fn deserialize(reader: &mut RefCursor<[u8]>, cp: &MergedCpLoad) -> EditorResult<Self> {
        Ok(if cp.cp3.pos_extended() {
            Self::Xyz(deserialize_vector::<3>(
                reader,
                cp.cp3.pos_format(),
                VectorDivisor::Custom(cp.cp3.pos_divisor()),
            )?)
        } else {
            Self::Xy(deserialize_vector::<2>(
                reader,
                cp.cp3.pos_format(),
                VectorDivisor::Custom(cp.cp3.pos_divisor()),
            )?)
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PositionData {
    NotPresent,
    Index8(u8),
    Index16(u16),
    Direct(DirectPosition),
}

impl PositionData {
    pub fn deserialize(reader: &mut RefCursor<[u8]>, cp: &MergedCpLoad) -> EditorResult<Self> {
        let pos_storage = cp.cp1.pos_storage();
        Ok(match pos_storage {
            VectorStorage::NotPresent => PositionData::NotPresent,
            VectorStorage::Index8 => PositionData::Index8(reader.read_u8()?),
            VectorStorage::Index16 => PositionData::Index16(reader.read_u16::<BigEndian>()?),
            VectorStorage::Direct => PositionData::Direct(DirectPosition::deserialize(reader, cp)?),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectNormal {
    /// Stores only the normal
    Single([f32; 3]),
    /// Store the normal, binormal and tangent
    Triple([f32; 9]),
}

impl DirectNormal {
    pub fn deserialize(reader: &mut RefCursor<[u8]>, cp: &MergedCpLoad) -> EditorResult<Self> {
        Ok(if cp.cp3.norm_extended() {
            Self::Triple(deserialize_vector::<9>(
                reader,
                VertexFormat::from(cp.cp3.norm_format()),
                VectorDivisor::Normalize,
            )?)
        } else {
            Self::Single(deserialize_vector::<3>(
                reader,
                VertexFormat::from(cp.cp3.norm_format()),
                VectorDivisor::Normalize,
            )?)
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NormalData {
    NotPresent,
    Index8(u8),
    Index16(u16),
    Direct(DirectNormal),
}

impl NormalData {
    pub fn deserialize(reader: &mut RefCursor<[u8]>, cp: &MergedCpLoad) -> EditorResult<Self> {
        let norm_storage = cp.cp1.norm_storage();
        Ok(match norm_storage {
            VectorStorage::NotPresent => NormalData::NotPresent,
            VectorStorage::Index8 => NormalData::Index8(reader.read_u8()?),
            VectorStorage::Index16 => NormalData::Index16(reader.read_u16::<BigEndian>()?),
            VectorStorage::Direct => NormalData::Direct(DirectNormal::deserialize(reader, cp)?),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectColor {
    AlphaDisabled(glam::U8Vec3),
    AlphaEnabled(glam::U8Vec4),
}

impl DirectColor {
    pub fn deserialize_col0(reader: &mut RefCursor<[u8]>, cp: &MergedCpLoad) -> EditorResult<Self> {
        Ok(if cp.cp3.col0_extended() {
            Self::AlphaEnabled(deserialize_color(reader, cp.cp3.col0_format())?)
        } else {
            Self::AlphaDisabled(deserialize_color(reader, cp.cp3.col0_format())?.xyz())
        })
    }

    pub fn deserialize_col1(reader: &mut RefCursor<[u8]>, cp: &MergedCpLoad) -> EditorResult<Self> {
        Ok(if cp.cp3.col1_extended() {
            Self::AlphaEnabled(deserialize_color(reader, cp.cp3.col1_format())?)
        } else {
            Self::AlphaDisabled(deserialize_color(reader, cp.cp3.col1_format())?.xyz())
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorData {
    NotPresent,
    Index8(u8),
    Index16(u16),
    Direct(DirectColor),
}

impl ColorData {
    pub fn deserialize_col0(reader: &mut RefCursor<[u8]>, cp: &MergedCpLoad) -> EditorResult<Self> {
        let color_storage = cp.cp1.col0_storage();
        Ok(match color_storage {
            VectorStorage::NotPresent => Self::NotPresent,
            VectorStorage::Index8 => Self::Index8(reader.read_u8()?),
            VectorStorage::Index16 => Self::Index16(reader.read_u16::<BigEndian>()?),
            VectorStorage::Direct => Self::Direct(DirectColor::deserialize_col0(reader, cp)?),
        })
    }

    pub fn deserialize_col1(reader: &mut RefCursor<[u8]>, cp: &MergedCpLoad) -> EditorResult<Self> {
        let color_storage = cp.cp1.col1_storage();
        Ok(match color_storage {
            VectorStorage::NotPresent => Self::NotPresent,
            VectorStorage::Index8 => Self::Index8(reader.read_u8()?),
            VectorStorage::Index16 => Self::Index16(reader.read_u16::<BigEndian>()?),
            VectorStorage::Direct => Self::Direct(DirectColor::deserialize_col1(reader, cp)?),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectUv {
    S(f32),
    St([f32; 2]),
}

#[derive(Debug, Clone, PartialEq)]
pub enum UvData {
    NotPresent,
    Index8(u8),
    Index16(u16),
    Direct(DirectUv),
}

/// Implements the deserialisation methods for all 7 UV fields.
// This is incredibly overengineered, but oh well.
macro_rules! impl_uv_de {
    // $id is the id of the UV (ranging from 0-7)
    //
    // $cp1 is the subcommand that contains the format and extended flag.
    // $cp2 contains the divisor.
    // For all, except uv4, these are equal.
    ($($id:literal => $cp1:literal + $cp2:literal),*) => {
        paste::paste! {
            impl DirectUv {
                $(
                    pub fn [< deserialize_uv $id >](reader: &mut RefCursor<[u8]>, cp: &MergedCpLoad) -> EditorResult<Self> {
                        let format = cp.[< cp $cp1 >].[< uv $id _format >]();
                        let divisor = cp.[< cp $cp2 >].[< uv $id _divisor >]();

                        Ok(if cp.[< cp $cp1 >].[< uv $id _extended >]() {
                            Self::St(deserialize_vector::<2>(reader, format, VectorDivisor::Custom(divisor))?)
                        } else {
                            Self::S(deserialize_scalar(reader, format, VectorDivisor::Custom(divisor))?)
                        })
                    }
                )*
            }

            impl UvData {
                $(
                    pub fn [< deserialize_uv $id >](reader: &mut RefCursor<[u8]>, cp: &MergedCpLoad) -> EditorResult<Self> {
                        let uv_storage = cp.cp2.[< uv $id _storage >]();
                        Ok(match uv_storage {
                            VectorStorage::NotPresent => Self::NotPresent,
                            VectorStorage::Index8 => Self::Index8(reader.read_u8()?),
                            VectorStorage::Index16 => Self::Index16(reader.read_u16::<BigEndian>()?),
                            VectorStorage::Direct => Self::Direct(DirectUv::[< deserialize_uv $id >](reader, cp)?)
                        })
                    }
                )*
            }
        }
    };
}

impl_uv_de!(0 => 3 + 3, 1 => 4 + 4, 2 => 4 + 4, 3 => 4 + 4, 4 => 4 + 5, 5 => 5 + 5, 6 => 5 + 5, 7 => 5 + 5);

#[derive(Debug, Clone, PartialEq)]
pub struct OpVertex {
    pub pm: Option<u8>,
    pub tms: [Option<u8>; 8],
    pub position: PositionData,
    pub normals: NormalData,
    pub color0: ColorData,
    pub color1: ColorData,
    pub uvs: [UvData; 8],
}

impl OpVertex {
    pub fn deserialize(reader: &mut RefCursor<[u8]>, cp: &MergedCpLoad) -> EditorResult<Self> {
        let pm = cp.cp1.pm().then(|| reader.read_u8()).transpose()?;
        let tms = [
            cp.cp1.tm0().then(|| reader.read_u8()).transpose()?,
            cp.cp1.tm1().then(|| reader.read_u8()).transpose()?,
            cp.cp1.tm2().then(|| reader.read_u8()).transpose()?,
            cp.cp1.tm3().then(|| reader.read_u8()).transpose()?,
            cp.cp1.tm4().then(|| reader.read_u8()).transpose()?,
            cp.cp1.tm5().then(|| reader.read_u8()).transpose()?,
            cp.cp1.tm6().then(|| reader.read_u8()).transpose()?,
            cp.cp1.tm7().then(|| reader.read_u8()).transpose()?,
        ];

        let position = PositionData::deserialize(reader, cp)?;
        let normals = NormalData::deserialize(reader, cp)?;
        let color0 = ColorData::deserialize_col0(reader, cp)?;
        let color1 = ColorData::deserialize_col1(reader, cp)?;

        let uvs = [
            UvData::deserialize_uv0(reader, cp)?,
            UvData::deserialize_uv1(reader, cp)?,
            UvData::deserialize_uv2(reader, cp)?,
            UvData::deserialize_uv3(reader, cp)?,
            UvData::deserialize_uv4(reader, cp)?,
            UvData::deserialize_uv5(reader, cp)?,
            UvData::deserialize_uv6(reader, cp)?,
            UvData::deserialize_uv7(reader, cp)?,
        ];

        Ok(OpVertex {
            pm,
            tms,
            position,
            normals,
            color0,
            color1,
            uvs,
        })
    }
}

/// Raw drawing commands.
///
/// These are directly for the Wii, not suitable for PC rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawOpCode {
    pub vertices: Vec<OpVertex>,
}

impl DrawOpCode {
    /// Decodes the draw command based on the vertex info given in the `LoadCP` opcode.
    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        cp_opcodes: &MergedCpLoad,
    ) -> EditorResult<Self> {
        let vertex_count = reader.read_u16::<BigEndian>()?;
        tracing::trace!("deserializing {vertex_count} vertices");

        let mut vertices = Vec::with_capacity(vertex_count as usize);
        for _ in 0..vertex_count {
            vertices.push(OpVertex::deserialize(reader, cp_opcodes)?);
        }

        Ok(Self { vertices })
    }
}
