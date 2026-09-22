use byteorder::{BigEndian, ReadBytesExt};

use crate::{error::EditorResult, format::encoding::Deserialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedLoad {
    /// The XF slot to pull the data from.
    pub index: u32,
}

impl Deserialize for IndexedLoad {
    fn deserialize(reader: &mut crate::shared::util::RefCursor<[u8]>) -> EditorResult<Self> {
        let index = reader.read_u32::<BigEndian>()?;
        Ok(Self { index })
    }
}
