use byteorder::{BigEndian, ReadBytesExt};

use crate::error::{CorruptionError, EditorError, EditorResult};
use crate::{
    format::encoding::{Deserialize, ReadArrayExt},
    shared::util::RefCursor,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentFormat {
    Uint8 = 0,
    Int8 = 1,
    Uint16 = 2,
    Int16 = 3,
    Float = 4,
}

impl TryFrom<u32> for ComponentFormat {
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

impl Deserialize for ComponentFormat {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

/// Deserializes vertex or normal components.
pub fn deserialize_components<const N: usize>(
    reader: &mut RefCursor<[u8]>,
    count: u16,
    format: ComponentFormat,
    divisor: u8,
) -> EditorResult<Vec<[f32; N]>> {
    let mut components = Vec::with_capacity(count as usize);
    let factor = 1.0 / 2.0f32.powi(divisor as i32);

    match format {
        ComponentFormat::Uint8 => {
            for _ in 0..count {
                let raw_comps = reader.read_u8_array::<N>()?;
                let comps = std::array::from_fn(|i| raw_comps[i] as f32 * factor);

                components.push(comps);
            }
        }
        ComponentFormat::Int8 => {
            for _ in 0..count {
                let raw_comps = reader.read_i8_array::<N>()?;
                let comps = std::array::from_fn(|i| raw_comps[i] as f32 * factor);

                components.push(comps);
            }
        }
        ComponentFormat::Uint16 => {
            for _ in 0..count {
                let raw_comps = reader.read_u16_array::<N, BigEndian>()?;
                let comps = std::array::from_fn(|i| raw_comps[i] as f32 * factor);

                components.push(comps);
            }
        }
        ComponentFormat::Int16 => {
            for _ in 0..count {
                let raw_comps = reader.read_i16_array::<N, BigEndian>()?;
                let comps = std::array::from_fn(|i| raw_comps[i] as f32 * factor);

                components.push(comps);
            }
        }
        ComponentFormat::Float => {
            for _ in 0..count {
                let raw_comps = reader.read_f32_array::<N, BigEndian>()?;

                components.push(raw_comps);
            }
        }
    }

    Ok(components)
}
