use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    chr0::Chr0Subfile,
    encoding::{Decode, Encode, ReadArrayExt, ReadStringExt},
    error::{
        CorruptionError, EncodingError, EncodingResult, IncorrectFormat, RangeError,
        UnsupportedError,
    },
    mdl0::Mdl0Subfile,
    pat0::Pat0Subfile,
};

const BRRES_MAGIC: [u8; 4] = [0x62, 0x72, 0x65, 0x73];
const LE_BOM: [u8; 2] = [0xFF, 0xFE];
const BE_BOM: [u8; 2] = [0xFE, 0xFF];

/// Returns the amount of sections a subfile has, which depends on the subfile type and version.
///
/// This info comes from [`BRRES Subfiles (File Format)`](https://mkwiiki.org/wiki/BRRES_Subfiles_(File_Format))
pub fn get_section_count(ty: SubfileType, version: u32) -> EncodingResult<usize> {
    Ok(match ty {
        SubfileType::Root => 0,
        SubfileType::Mdl0 => match version {
            8 => 11,
            11 => 14,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid MDL0 version: {version} (must be 8, 11)"),
                    ..Default::default()
                }
                .into());
            }
        },
        SubfileType::Chr0 => match version {
            // 3 => 1,
            3 => {
                return Err(UnsupportedError {
                    reason: "CHR0 version 3".to_owned(),
                    ..Default::default()
                }
                .into());
            }
            5 => 2,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid CHR0 version: {version} (must be 3, 5)"),
                    ..Default::default()
                }
                .into());
            }
        },
        SubfileType::Pat0 => match version {
            4 => 6,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid PAT0 version: {version} (must be 4)"),
                    ..Default::default()
                }
                .into());
            }
        },
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Header {
    pub size: u32,
    pub root_offset: u16,
    pub section_count: u16,
}

impl Decode for Header {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let magic = reader.read_u8_array::<4>()?;
        if magic != BRRES_MAGIC {
            return Err(IncorrectFormat {
                expected_magic: BRRES_MAGIC.to_vec(),
                found_magic: magic.to_vec(),
                location: Some(reader.position()),
            }
            .into());
        }

        let is_be = match reader.read_u8_array::<2>()? {
            BE_BOM => true,
            LE_BOM => false,
            bom => {
                return Err(CorruptionError {
                    reason: format!(
                        "byte order mark is incorrect, expected FEFF or FFFE, found {bom:x?}"
                    ),
                    ..Default::default()
                }
                .into());
            }
        };

        if !is_be {
            return Err(UnsupportedError {
                reason: "little endian brres files are not supported".to_owned(),
                ..Default::default()
            }
            .into());
        }

        let _padding = reader.read_u16::<BigEndian>()?;
        let size = reader.read_u32::<BigEndian>()?;
        let root_offset = reader.read_u16::<BigEndian>()?;
        let section_count = reader.read_u16::<BigEndian>()?;

        Ok(Self {
            root_offset,
            size,
            section_count,
        })
    }
}

pub trait Subfile {
    const MAGIC: [u8; 4];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootSubfile {
    pub size: u32,
}

impl Decode for RootSubfile {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let magic = reader.read_u8_array::<4>()?;
        if magic != Self::MAGIC {
            return Err(IncorrectFormat {
                expected_magic: Self::MAGIC.to_vec(),
                found_magic: magic.to_vec(),
                location: Some(reader.position()),
            }
            .into());
        }

        Ok(RootSubfile {
            size: reader.read_u32::<BigEndian>()?,
        })
    }
}

impl Subfile for RootSubfile {
    const MAGIC: [u8; 4] = [0x72, 0x6f, 0x6f, 0x74]; // "root"
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubfileHeader {
    /// Start position of this header. This is used to compute subfile section positions using their
    /// offsets.
    pub header_start: u32,
    /// Length of this subfile.
    pub subfile_length: u32,
    /// Version of ths subfile. For MDL0 this is either 8 or 11.
    pub subfile_version: u32,
    /// The offset to the outer BRRES file.
    pub brres_offset: i32,
    /// Offsets within this BRRES file. The number of offsets is implied by the version.
    /// The number can be obtained using the [`get_section_count`] function.
    pub offsets: Vec<i32>,
    /// String offset to the name of this subfile.
    /// This offset is relative to [`header_start`](Self::header_start).
    ///
    /// Note that the offset points to the start of the string data, the length prefix is 4 bytes ahead of it.
    pub name_offset: i32,
}

impl SubfileHeader {
    pub fn decode(reader: &mut Cursor<&[u8]>, ty: SubfileType) -> EncodingResult<Self> {
        let header_start = reader.position() as u32 - 4; // Subtract 4 for magic.
        let subfile_length = reader.read_u32::<BigEndian>()?;
        let subfile_version = reader.read_u32::<BigEndian>()?;
        let brres_offset = reader.read_i32::<BigEndian>()?;

        let section_count = get_section_count(ty, subfile_version)?;

        let mut offsets = Vec::with_capacity(section_count);
        for _ in 0..section_count {
            offsets.push(reader.read_i32::<BigEndian>()?);
        }

        let name_offset = reader.read_i32::<BigEndian>()?;

        Ok(Self {
            header_start,
            subfile_length,
            subfile_version,
            brres_offset,
            offsets,
            name_offset,
        })
    }

    /// Obtains the starting index of the specified section.
    pub fn get_section_start(&self, section_index: usize) -> EncodingResult<u32> {
        let offset = *self.offsets.get(section_index).ok_or_else(|| {
            EncodingError::from(RangeError {
                requested: section_index as u64,
                range: 0..self.offsets.len() as u64,
                ..Default::default()
            })
        })?;

        Ok((self.header_start as i32 + offset) as u32)
    }
}

macro_rules! impl_subfile_enum {
    ($($ty: ident),*) => {
        paste::paste! {
            #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
            pub enum SubfileType {
                $($ty),*
            }

            #[derive(Debug, Clone, PartialEq)]
            pub enum SubfileData {
                $($ty([< $ty Subfile >])),*
            }
        }
    }
}

impl_subfile_enum!(Root, Mdl0, Chr0, Pat0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexGroupHeader {
    pub length: u32,
    pub number: u32,
}

impl Decode for IndexGroupHeader {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        Ok(Self {
            length: reader.read_u32::<BigEndian>()?,
            number: reader.read_u32::<BigEndian>()?,
        })
    }
}

impl Encode for IndexGroupHeader {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        writer.write_u32::<BigEndian>(self.length)?;
        writer.write_u32::<BigEndian>(self.number)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexGroupEntry {
    pub entry_id: u16,
    pub flag: u16,
    pub left_index: u16,
    pub right_index: u16,
    pub name_pointer: u32,
    pub data_pointer: u32,
}

impl Decode for IndexGroupEntry {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let entry_id = reader.read_u16::<BigEndian>()?;
        let flag = reader.read_u16::<BigEndian>()?;
        let left_index = reader.read_u16::<BigEndian>()?;
        let right_index = reader.read_u16::<BigEndian>()?;
        let name_pointer = reader.read_u32::<BigEndian>()?;
        let data_pointer = reader.read_u32::<BigEndian>()?;

        Ok(Self {
            entry_id,
            flag,
            left_index,
            right_index,
            name_pointer,
            data_pointer,
        })
    }
}

impl Encode for IndexGroupEntry {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        writer.write_u16::<BigEndian>(self.entry_id)?;
        writer.write_u16::<BigEndian>(self.flag)?;
        writer.write_u16::<BigEndian>(self.left_index)?;
        writer.write_u16::<BigEndian>(self.right_index)?;
        writer.write_u32::<BigEndian>(self.name_pointer)?;
        writer.write_u32::<BigEndian>(self.data_pointer)?;

        Ok(())
    }
}

/// Describes locations of the subfiles in this BRRES file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexGroup {
    /// Index into the BRRES file where this group starts. Generally this is just right after the BRRES header.
    pub group_start: u32,
    /// The index group header.
    pub header: IndexGroupHeader,
    /// `header.number + 1` entries.
    ///
    /// The first entry is a dummy entry that has no name.
    pub entries: Vec<IndexGroupEntry>,
}

impl IndexGroup {
    /// Obtains the name of the given index group entry using its name pointer.
    ///
    /// `data` should be the entire BRRES file (including header).
    ///
    /// # Conditions
    /// - The given index group entry must be owned by the current index group.
    ///
    /// Violating these conditions will not cause unsoundness but will either cause a panic due to invalid
    /// UTF-8 or return incorrect strings.
    pub fn get_entry_name<'pool>(
        &self,
        data: &'pool [u8],
        entry: &IndexGroupEntry,
    ) -> EncodingResult<&'pool str> {
        if entry.name_pointer == 0 {
            return Ok(""); // This entry has no name.
        }

        // Move cursor to name and then back after reading.
        let name_start = self.group_start + entry.name_pointer;
        let mut reader = Cursor::new(&data[name_start as usize - 4..]);
        let name = reader.read_u32_str::<BigEndian>()?;

        // Confirm both the null terminated and length prefixed strings are equal.
        // This is an extra check to ensure offsets are correct.
        #[cfg(debug_assertions)]
        {
            reader.set_position(4); // Skip length prefix.
            let null_name = reader.read_null_str::<BigEndian>()?;

            debug_assert_eq!(
                null_name, name,
                "prefixed and null-terminated names are not equal"
            );
        }

        Ok(name)
    }

    pub fn get_entry_data_start(&self, entry: &IndexGroupEntry) -> u32 {
        self.group_start + entry.data_pointer
    }
}

impl Decode for IndexGroup {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let group_start = reader.position() as u32;
        let header = IndexGroupHeader::decode(reader)?;

        let mut entries = Vec::with_capacity(header.number as usize);
        for _ in 0..header.number + 1 {
            let entry = IndexGroupEntry::decode(reader)?;
            entries.push(entry);
        }

        debug_assert_eq!(
            reader.position() as u32 - group_start,
            header.length,
            "an incorrect number of index group entries was read"
        );

        Ok(Self {
            group_start,
            header,
            entries,
        })
    }
}

impl Encode for IndexGroup {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        self.header.encode_into(writer)?;
        for entry in &self.entries {
            entry.encode_into(writer)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArchiveFolder {
    pub name: String,
    pub files: Vec<ArchiveFile>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArchiveFile {
    pub name: String,
    pub file: SubfileData,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Archive {
    pub folders: Vec<ArchiveFolder>,
}

impl Archive {
    fn decode_subfile(
        reader: &mut Cursor<&[u8]>,
        _index: &IndexGroupEntry,
    ) -> EncodingResult<SubfileData> {
        let magic = reader.read_u8_array::<4>()?;
        Ok(match magic {
            Mdl0Subfile::MAGIC => SubfileData::Mdl0(Mdl0Subfile::decode(reader)?),
            Pat0Subfile::MAGIC => SubfileData::Pat0(Pat0Subfile::decode(reader)?),
            Chr0Subfile::MAGIC => SubfileData::Chr0(Chr0Subfile::decode(reader)?),
            _ => {
                return Err(UnsupportedError {
                    reason: format!("file type `{}`", String::from_utf8_lossy(&magic)),
                    location: Some(reader.position()),
                }
                .into());
            }
        })
    }
}

impl Decode for Archive {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let header = Header::decode(reader)?;

        // Skip to root start
        reader.set_position(header.root_offset as u64);

        let _root = RootSubfile::decode(reader)?;

        // Root group, which contains folders such as AnmChr(NW4R) or AnmTexPat(NW4R).
        let root_group = IndexGroup::decode(reader)?;

        let mut folders = Vec::with_capacity(root_group.entries.len() - 1);
        for folder in &root_group.entries[1..] {
            let folder_name = root_group.get_entry_name(reader.get_ref(), folder)?;

            tracing::trace!(
                "Discovered folder `{folder_name}` at location {}",
                reader.position()
            );

            reader.set_position(root_group.get_entry_data_start(folder) as u64);

            // Folder group which contains the actual subfiles.
            let child_group = IndexGroup::decode(reader)?;
            let mut subfiles = Vec::with_capacity(child_group.entries.len() - 1);

            for file in &child_group.entries[1..] {
                let file_name = child_group.get_entry_name(reader.get_ref(), file)?;

                tracing::trace!(
                    "Discovered file `{folder_name}/{file_name}` at location {}",
                    reader.position()
                );

                reader.set_position(child_group.get_entry_data_start(file) as u64);

                tracing::trace!(
                    "Reading `{folder_name}/{file_name}` data section at location {}",
                    reader.position()
                );

                let file = tracing::trace_span!("decode_subfile", %folder_name, %file_name)
                    .in_scope(|| Self::decode_subfile(reader, file))?;

                subfiles.push(ArchiveFile {
                    name: file_name.to_owned(),
                    file,
                });
            }

            folders.push(ArchiveFolder {
                name: folder_name.to_owned(),
                files: subfiles,
            });
        }

        Ok(Self { folders })
    }
}
