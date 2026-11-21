use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use slipstream_derive::try_from;
use slipstream_shared::{
    RefCursor, SlipstreamResult, cursor::MutCursor, error::CorruptionError, verify,
};
use smallvec::SmallVec;

use crate::encoding::ReadArrayExt;

pub const BMG_MAGIC: [u8; 4] = *b"MESG";
pub const BMG_DATA_TYPE: [u8; 4] = *b"bmg1";

/// The character encoding used for the text in a BMG file.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[try_from(u8)]
pub enum Charset {
    Undefined = 0x00,
    /// Single-byte CP1251 encoding.
    Cp1251 = 0x01,
    Utf16 = 0x02,
    /// Shift-JIS encoding.
    Sjis = 0x03,
    Utf8 = 0x04,
}

impl Charset {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let byte = reader.read_u8()?;
        Self::try_from(byte)
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u8(*self as u8)?;
        Ok(())
    }
}

/// A selection of fonts from `Font.szs`.
///
/// This is used in BMG files to select a font for the messages.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[try_from(u8)]
pub enum FontKind {
    /// `tt_kart_font_rodan_ntlg_pro_b.brfnt`.
    ///
    /// Used for the countdown and for race strings like `FINISH!`.
    RodanNtlgPro = 0x00,
    /// `kart_kanji_font.brfnt` or `kart_font_korea.brfnt` in the NTSC-K version.
    ///
    /// Used for nearly all messages.
    KanjiOrKorea = 0x01,
    /// `tt_kart_extension_font.brfnt`.
    ///
    /// Unused.
    Extension = 0x02,
    /// `indictator_font.brfnt`.
    ///
    /// Unused.
    Indicator = 0x03,
    /// `mario_font_number_red.brfnt`.
    ///
    /// Special red font (only digits) used for battle and team points.
    FontNumberRed = 0x04,
    /// `mario_font_number_blue.brfnt`.
    ///
    /// Special blue font (only digits) used for battle and team points
    FontNumberBlue = 0x05,
}

impl FontKind {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let byte = reader.read_u8()?;
        Self::try_from(byte)
    }
}

/// The text index attributes format specifically for Mario Kart Wii.
///
/// Other games will have different formats.
#[derive(Debug, Clone, PartialEq)]
pub struct MkTextIndexAttributes {
    pub font: FontKind,
}

impl MkTextIndexAttributes {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let font = FontKind::deserialize(reader)?;
        Ok(Self { font })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextIndexAttributes {
    MkWii(MkTextIndexAttributes),
    Unknown(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextIndexTableEntry {
    /// Offset into the text pool of the `DAT1` section.
    pub text_offset: u32,
    /// Attributes for the table entry.
    ///
    /// This is game-dependent binary data. Only the Mario Kart Wii is parsed currently.
    pub attributes: TextIndexAttributes,
}

impl TextIndexTableEntry {
    pub fn deserialize(reader: &mut RefCursor<[u8]>, entry_size: u16) -> SlipstreamResult<Self> {
        let text_offset = reader.read_u32::<BigEndian>()?;

        let before_attr = reader.position();

        // Check whether these attributes match the ones we would expect in Mario Kart Wii.

        let attributes = reader.slice(0..entry_size as usize)?;
        let attributes = if Self::is_mkwii(attributes) {
            TextIndexAttributes::MkWii(MkTextIndexAttributes::deserialize(reader)?)
        } else {
            TextIndexAttributes::Unknown(attributes.to_vec())
        };

        // Ensure we skipped to the end of the attributes.
        reader.set_position(before_attr + entry_size as u64);

        Ok(Self {
            text_offset,
            attributes,
        })
    }

    /// Heuristic to detect whether this is a BMG attribute for Mario Kart Wii
    fn is_mkwii(attrs: &[u8]) -> bool {
        attrs.len() == 4 && attrs[0] <= 0x05
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextIndexTable {
    pub group_id: u16,
    pub entries: Vec<TextIndexTableEntry>,
}

impl TextIndexTable {
    pub const MAGIC: [u8; 4] = *b"INF1";

    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let num_entries = reader.read_u32::<BigEndian>()?;
        let entry_size = reader.read_u16::<BigEndian>()?;
        let group_id = reader.read_u16::<BigEndian>()?;
        let default_color = reader.read_u8()?;
        let _reserved = reader.read_u8()?;

        let mut entries = Vec::with_capacity(num_entries as usize);
        for _ in 0..num_entries {
            // Check whether these attributes are for Mario Kart Wii, otherwise we skip parsing them.
            entries.push(TextIndexTableEntry::deserialize(reader, entry_size)?);
        }

        Ok(Self { entries, group_id })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BmgSection {
    IndexTable(TextIndexTable),
}

impl BmgSection {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let magic = reader.read_u8_array::<4>()?;
        let len = reader.read_u32::<BigEndian>()?;

        let data = match magic {
            TextIndexTable::MAGIC => Self::IndexTable(TextIndexTable::deserialize(reader)?),
            _ => {
                return Err(CorruptionError {
                    reason: format!(
                        "invalid BMG section magic: `{}`",
                        String::from_utf8_lossy(&magic)
                    ),
                    location: Some(reader.position()),
                }
                .into());
            }
        };
        Ok(data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BmgHeader {
    pub len: u32,
    pub num_sections: u32,
    pub charset: Charset,
    pub sections: SmallVec<[BmgSection; 3]>,
}

impl BmgHeader {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let signature = reader.read_u8_array::<4>()?;
        verify!(
            signature == BMG_MAGIC,
            "Expected BMG magic `MESG`, got {}",
            String::from_utf8_lossy(&signature)
        );

        let data_type = reader.read_u8_array::<4>()?;
        verify!(
            data_type == BMG_DATA_TYPE,
            "Expected BMG data type `bmg1`, got {}",
            String::from_utf8_lossy(&data_type)
        );

        let len = reader.read_u32::<BigEndian>()?;
        let num_sections = reader.read_u32::<BigEndian>()?;
        let charset = Charset::deserialize(reader)?;

        let _reserved0 = reader.read_u8()?;
        let _reserved1 = reader.read_u16::<BigEndian>()?;
        let _reserved2 = reader.read_i32_array::<2, BigEndian>()?;
        let _user_work = reader.read_i32::<BigEndian>()?;

        let mut sections = SmallVec::new();
        for i in 0..num_sections {
            sections.push(BmgSection::deserialize(reader)?);
        }

        Ok(Self {
            len,
            num_sections,
            charset,
            sections,
        })
    }
}

pub struct Bmg {}

impl Bmg {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        todo!()
    }
}
