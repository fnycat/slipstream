use byteorder::{BigEndian, ReadBytesExt};

use crate::{error::EditorResult, format::encoding::Deserialize, shared::util::RefCursor};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallDisplayList {
    pub address: u32,
    pub size: u32,
}

impl Deserialize for CallDisplayList {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let address = reader.read_u32::<BigEndian>()?;
        let size = reader.read_u32::<BigEndian>()?;

        Ok(Self { address, size })
    }
}
