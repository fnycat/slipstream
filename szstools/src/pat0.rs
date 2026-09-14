use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    brres::{IndexGroup, IndexGroupEntry, Subfile, SubfileHeader, SubfileType},
    encoding::{Deserialize, Encode, ReadStringExt},
    error::EncodingResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pat0Header {
    pub frame_count: u16,
    pub base_number: u16,
    pub string_number: u16,
    pub cyclic: bool,
}

impl Deserialize for Pat0Header {
    fn deserialize(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
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
            cyclic,
        })
    }
}

impl Encode for Pat0Header {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        writer.write_u32::<BigEndian>(0)?; // Unknown
        writer.write_u16::<BigEndian>(self.frame_count)?;
        writer.write_u16::<BigEndian>(self.base_number)?;
        writer.write_u16::<BigEndian>(self.string_number)?;
        writer.write_u32::<BigEndian>(0)?; // Unknown
        writer.write_u16::<BigEndian>(self.cyclic as u16)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationSection {}

/// Both section #1 and section #3 have the same format of `string_number` amount of u32s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct U32Section {
    pub offsets: Vec<u32>,
}

impl U32Section {
    pub fn deserialize(reader: &mut Cursor<&[u8]>, string_number: u16) -> EncodingResult<Self> {
        let mut offsets = Vec::with_capacity(string_number as usize);
        for _ in 0..string_number {
            offsets.push(reader.read_u32::<BigEndian>()?);
        }

        Ok(Self { offsets })
    }
}

impl Encode for U32Section {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        for &offset in &self.offsets {
            writer.write_u32::<BigEndian>(offset);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pat0Subfile {
    pub subfile_header: SubfileHeader,
    pub pat0_header: Pat0Header,
}

impl Deserialize for Pat0Subfile {
    fn deserialize(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let subfile_header = SubfileHeader::deserialize(reader, SubfileType::Pat0)?;
        let pat0_header = Pat0Header::deserialize(reader)?;

        let mut name_start = Cursor::new(
            &reader.get_ref()
                [subfile_header.header_start as usize + subfile_header.name_offset as usize..],
        );

        let pat0_name = name_start.read_null_str::<BigEndian>()?;
        dbg!(pat0_name);

        dbg!(&subfile_header, pat0_header);

        let index_group = IndexGroup::deserialize(reader)?;
        let name = index_group.get_entry_name(reader.get_ref(), &index_group.entries[1])?;
        dbg!(name);

        todo!();
    }
}

impl Subfile for Pat0Subfile {
    const MAGIC: [u8; 4] = [0x50, 0x41, 0x54, 0x30]; // "PAT0"
}
