use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use slipstream_derive::Inspect;
use slipstream_shared::cursor::{MutCursor, RefCursor, SizeEstimate};
use slipstream_shared::error::SlipstreamResult;

#[derive(Debug, Default, Clone, PartialEq, Eq, Inspect)]
pub struct CallDisplayList {
    pub address: u32,
    pub size: u32
}

impl CallDisplayList {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let address = reader.read_u32::<BigEndian>()?;
        let size = reader.read_u32::<BigEndian>()?;

        Ok(Self { address, size })
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(self.address)?;
        writer.write_u32::<BigEndian>(self.size)?;
        Ok(())
    }
}

impl SizeEstimate for CallDisplayList {
    #[inline]
    fn estimate_size(&self) -> usize {
        8
    }
}
