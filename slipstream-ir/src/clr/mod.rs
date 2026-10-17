use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::node::arena::{IrArena, IrNodeKey};

pub fn deserialize(
    reader: &mut RefCursor<[u8]>,
    parent: Option<IrNodeKey>,
    arena: &IrArena,
    label: String
) -> SlipstreamResult<IrNodeKey> {
    todo!()
}
