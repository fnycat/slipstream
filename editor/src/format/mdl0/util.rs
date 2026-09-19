use byteorder::{BigEndian, ReadBytesExt};

use crate::error::{CorruptionError, EditorError, EditorResult};
use crate::{
    format::encoding::{Deserialize, ReadArrayExt},
    shared::util::RefCursor,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VectorPrecision {
    Uint8 = 0,
    Int8 = 1,
    Uint16 = 2,
    Int16 = 3,
    Float = 4,
}

impl TryFrom<u32> for VectorPrecision {
    type Error = EditorError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Uint8,
            1 => Self::Int8,
            2 => Self::Uint16,
            3 => Self::Int16,
            4 => Self::Float,
            v => {
                return Err(CorruptionError {
                    reason: format!("invalid vertex format: {v} (expected 0-4)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

impl Deserialize for VectorPrecision {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

pub fn deserialize_scalar_data(
    reader: &mut RefCursor<[u8]>,
    count: u16,
    format: VectorPrecision,
    divisor: u8,
) -> EditorResult<Vec<f32>> {
    let mut components = Vec::with_capacity(count as usize);
    let factor = 1.0 / 2.0f32.powi(divisor as i32);

    match format {
        VectorPrecision::Uint8 => {
            for _ in 0..count {
                let raw = reader.read_u8()?;
                components.push(raw as f32 * factor);
            }
        }
        VectorPrecision::Int8 => {
            for _ in 0..count {
                let raw = reader.read_i8()?;
                components.push(raw as f32 * factor);
            }
        }
        VectorPrecision::Uint16 => {
            for _ in 0..count {
                let raw = reader.read_u16::<BigEndian>()?;
                components.push(raw as f32 * factor);
            }
        }
        VectorPrecision::Int16 => {
            for _ in 0..count {
                let raw = reader.read_i16::<BigEndian>()?;
                components.push(raw as f32 * factor);
            }
        }
        VectorPrecision::Float => {
            for _ in 0..count {
                let raw = reader.read_f32::<BigEndian>()?;
                components.push(raw);
            }
        }
    }

    Ok(components)
}

/// Deserializes vertex, normal or UV components.
pub fn deserialize_vector_data<const N: usize>(
    reader: &mut RefCursor<[u8]>,
    count: u16,
    format: VectorPrecision,
    divisor: u8,
) -> EditorResult<Vec<[f32; N]>> {
    let mut components = Vec::with_capacity(count as usize);
    let factor = 1.0 / 2.0f32.powi(divisor as i32);

    match format {
        VectorPrecision::Uint8 => {
            for _ in 0..count {
                let raw_comps = reader.read_u8_array::<N>()?;
                let comps = std::array::from_fn(|i| raw_comps[i] as f32 * factor);

                components.push(comps);
            }
        }
        VectorPrecision::Int8 => {
            for _ in 0..count {
                let raw_comps = reader.read_i8_array::<N>()?;
                let comps = std::array::from_fn(|i| raw_comps[i] as f32 * factor);

                components.push(comps);
            }
        }
        VectorPrecision::Uint16 => {
            for _ in 0..count {
                let raw_comps = reader.read_u16_array::<N, BigEndian>()?;
                let comps = std::array::from_fn(|i| raw_comps[i] as f32 * factor);

                components.push(comps);
            }
        }
        VectorPrecision::Int16 => {
            for _ in 0..count {
                let raw_comps = reader.read_i16_array::<N, BigEndian>()?;
                let comps = std::array::from_fn(|i| raw_comps[i] as f32 * factor);

                components.push(comps);
            }
        }
        VectorPrecision::Float => {
            for _ in 0..count {
                let raw_comps = reader.read_f32_array::<N, BigEndian>()?;

                components.push(raw_comps);
            }
        }
    }

    Ok(components)
}
