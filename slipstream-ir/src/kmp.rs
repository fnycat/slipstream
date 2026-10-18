use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::brres::SubfileHeader;
use crate::node::arena::{IrArena, IrNodeKey};

pub const KMP_MAGIC: [u8; 4] = [0x52, 0x4b, 0x4d, 0x44]; // "RKMD"

pub struct KmpHeader {}

impl KmpHeader {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        todo!()
    }
}

pub fn deserialize(
    reader: &mut RefCursor<[u8]>,
    parent: Option<IrNodeKey>,
    arena: &IrArena,
    name: String
) -> SlipstreamResult<IrNodeKey> {
    todo!()
}
