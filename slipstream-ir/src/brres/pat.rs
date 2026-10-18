use byteorder::{BigEndian, ReadBytesExt};
use slipstream_shared::cursor::RefCursor;
use slipstream_shared::error::SlipstreamResult;

use crate::brres::{SubfileHeader, SubfileType};
use crate::encoding::ReadStringExt;
use crate::index::IndexGroup;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatHeader {
    pub frame_count: u16,
    pub base_number: u16,
    pub string_number: u16,
    pub cyclic: bool
}

impl PatHeader {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let _unknown12 = reader.read_u32::<BigEndian>()?; // 2 + 2 unknown bytes
        let frame_count = reader.read_u16::<BigEndian>()?;
        let base_number = reader.read_u16::<BigEndian>()?;
        let string_number = reader.read_u16::<BigEndian>()?;
        let _unknown34 = reader.read_u32::<BigEndian>()?; // 2 + 2 unknown bytes
        let cyclic = reader.read_u16::<BigEndian>()? != 0;

        Ok(Self {
            frame_count,
            base_number,
            string_number,
            cyclic
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationSection {}

/// Both section #1 and section #3 have the same format of `string_number` amount of u32s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct U32Section {
    pub offsets: Vec<u32>
}

impl U32Section {
    pub fn deserialize(reader: &mut RefCursor<[u8]>, string_number: u16) -> SlipstreamResult<Self> {
        let mut offsets = Vec::with_capacity(string_number as usize);
        for _ in 0..string_number {
            offsets.push(reader.read_u32::<BigEndian>()?);
        }

        Ok(Self { offsets })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternAnimation {
    pub subfile_header: SubfileHeader,
    pub pat0_header: PatHeader
}

impl PatternAnimation {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let subfile_header = SubfileHeader::deserialize(reader, SubfileType::Pat)?;
        let pat0_header = PatHeader::deserialize(reader)?;

        let name_start = subfile_header.header_start as i64 + subfile_header.name_offset as i64;
        reader.set_position(name_start as u64);

        let pat0_name = reader.read_null_string::<BigEndian>()?;
        dbg!(pat0_name);

        dbg!(&subfile_header, pat0_header);

        let index_group = IndexGroup::deserialize(reader)?;
        let name = index_group.get_entry_name(reader, &index_group.entries[1])?;
        dbg!(name);

        todo!();
    }
}
