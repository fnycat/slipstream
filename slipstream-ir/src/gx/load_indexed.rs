use bitfield_struct::bitfield;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use slipstream_shared::{
    cursor::{MutCursor, RefCursor, SizeEstimate},
    error::SlipstreamResult,
};

/// Loads a matrix from the table and puts it into the given slot.
#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct IndexedLoad {
    /// Address into the indexed array. This should be divided by 12 to get the XF slot number.
    /// This value will always be between 0 and 108 (i.e. there are 10 hardware XF slots)
    #[bits(12)]
    pub address: u16,
    /// Transfer count - 1. This indicates the amount of words to read from the the index.
    /// For example a 3x4 position matrix is 12 floats, therefore a position load will have `transfer_count` of 11.
    #[bits(4)]
    pub transfer_count: u8,
    /// The matrix slot to pull the data from
    #[bits(16)]
    pub index: u16,
}

impl IndexedLoad {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(self.into_bits())?;
        Ok(())
    }
}

impl SizeEstimate for IndexedLoad {
    #[inline]
    fn estimate_size(&self) -> usize {
        4
    }
}
