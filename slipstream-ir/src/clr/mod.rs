use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::brres::{SubfileHeader, SubfileType};
use crate::node::arena::{IrArena, IrNodeKey};

pub const CLR_MAGIC: [u8; 4] = [0x43, 0x4c, 0x52, 0x30]; // "CLR0"

#[derive(Debug, Clone, PartialEq)]
pub struct ClrHeader {}

impl ClrHeader {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        todo!()
    }
}

#[tracing::instrument(skip_all, fields(name, parent_id))]
pub fn deserialize(
    reader: &mut RefCursor<[u8]>,
    parent_id: Option<IrNodeKey>,
    arena: &IrArena,
    name: String
) -> SlipstreamResult<IrNodeKey> {
    let subfile_header = SubfileHeader::deserialize(reader, SubfileType::Clr);

    todo!()
}
