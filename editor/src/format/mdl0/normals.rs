use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt};

use crate::format::{
    encoding::Deserialize,
    error::{CorruptionError, EncodingResult},
    mdl0::{
        mdl0::SectionDeserialize,
        util::{ComponentFormat, deserialize_components},
    },
};

const COMPONENTS_NORMAL: u32 = 0x0;
const COMPONENTS_ALL: u32 = 0x1;
const COMPONENTS_ANY: u32 = 0x2;

#[derive(Debug, Clone, PartialEq)]
pub enum NormalData {
    /// Only the normal.
    Normal(Vec<[f32; 3]>),
    /// Includes all of the normal, bi-normal and tangent
    All(Vec<[f32; 9]>),
    /// Either the normal, bi-normal or tangent.
    Any(Vec<[f32; 3]>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Normals {
    pub header_start: u32,
    pub mdl0_offset: i32,
    pub data_offset: i32,
    pub name_offset: i32,
    pub index: u32,
    pub format: ComponentFormat,
    pub divisor: u8,
    pub stride: u8,
    pub normals: NormalData,
}

impl SectionDeserialize for Normals {
    fn deserialize_section(reader: &mut Cursor<&[u8]>, header_start: u32) -> EncodingResult<Self> {
        let _length = reader.read_u32::<BigEndian>()?;
        let mdl0_offset = reader.read_i32::<BigEndian>()?;
        let data_offset = reader.read_i32::<BigEndian>()?;
        let name_offset = reader.read_i32::<BigEndian>()?;
        let index = reader.read_u32::<BigEndian>()?;
        let component_count = reader.read_u32::<BigEndian>()?;
        let format = ComponentFormat::deserialize(reader)?;
        let divisor = reader.read_u8()?;
        let stride = reader.read_u8()?;
        let normal_count = reader.read_u16::<BigEndian>()?;

        let normals_start = header_start as i64 + data_offset as i64;
        reader.set_position(normals_start as u64);

        let normals = match component_count {
            COMPONENTS_NORMAL => NormalData::Normal(deserialize_components::<3>(
                reader,
                normal_count,
                format,
                divisor,
            )?),
            COMPONENTS_ALL => NormalData::All(deserialize_components::<9>(
                reader,
                normal_count,
                format,
                divisor,
            )?),
            COMPONENTS_ANY => NormalData::Any(deserialize_components::<3>(
                reader,
                normal_count,
                format,
                divisor,
            )?),
            v => {
                return Err(CorruptionError {
                    reason: format!("invalid component count: {v} (expected 0-2)"),
                    location: Some(reader.position()),
                    ..Default::default()
                }
                .into());
            }
        };

        Ok(Self {
            header_start,
            mdl0_offset,
            data_offset,
            name_offset,
            index,
            format,
            divisor,
            stride,
            normals,
        })
    }
}
