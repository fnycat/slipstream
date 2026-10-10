use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::{Inspect, try_from};
use slipstream_shared::{RefCursor, SlipstreamError, SlipstreamResult, error::IncorrectFormat};

use crate::{encoding::ReadArrayExt, node::arena::IrNodeKey};

pub const TPL_MAGIC: [u8; 4] = [0x00, 0x20, 0xAF, 0x30];

#[derive(Debug, Clone, PartialEq, Eq, Inspect)]
#[inspect(summary = "TplHeader::summary")]
pub struct TplHeader {
    #[inspect(tooltip = "Cursor position at the start of the header.")]
    pub header_start: u64,
    #[inspect(tooltip = "Number of images in this file.")]
    pub image_count: u32,
    #[inspect(tooltip = "Offset in bytes to the image table.")]
    pub table_offset: i32,
}

impl TplHeader {
    #[inline]
    pub fn summary(&self) -> String {
        format!("TPL Header: {} image(s)", self.image_count)
    }

    #[inline]
    pub fn get_table_start(&self) -> u64 {
        (self.header_start as i64 + self.table_offset as i64) as u64
    }

    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let header_start = reader.position();

        let magic = reader.read_u8_array::<4>()?;
        if magic != TPL_MAGIC {
            return Err(IncorrectFormat {
                expected_magic: TPL_MAGIC.into(),
                found_magic: magic.into(),
                location: Some(reader.position()),
            }
            .into());
        }

        let image_count = reader.read_u32::<BigEndian>()?;
        let table_offset = reader.read_i32::<BigEndian>()?;

        Ok(Self {
            header_start,
            image_count,
            table_offset,
        })
    }
}

#[try_from(u32)]
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteFormat {
    Ia8 = 0x00,
    Rgb565 = 0x01,
    Rgb5A3 = 0x02,
}

impl PaletteFormat {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaletteHeader {}

impl PaletteHeader {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let entry_count = reader.read_u16::<BigEndian>()?;
        let unpacked = reader.read_u8()?;
        let _padding = reader.read_u8()?;
        let palette_format = PaletteFormat::deserialize(reader)?;

        todo!()
    }
}

pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<IrNodeKey> {
    let header = TplHeader::deserialize(reader)?;
    reader.set_position(header.get_table_start());

    let mut next_offset_start = reader.position();

    let mut images = Vec::<PaletteHeader>::with_capacity(header.image_count as usize);
    for _ in 0..header.image_count {
        reader.set_position(next_offset_start);
        let image_offset = reader.read_i32::<BigEndian>()?;
        let palette_offset = reader.read_i32::<BigEndian>()?;
        next_offset_start = reader.position();
    }

    todo!()
}
