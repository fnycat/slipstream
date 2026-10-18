use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::brres::DeserializeContents;
use crate::node::node::IrNodeType;

pub struct FurVectors {}

impl DeserializeContents for FurVectors {
    const KIND: IrNodeType = IrNodeType::FurVectors;
    const NAME: &str = "Fur Vectors";

    fn deserialize_contents(_reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        tracing::warn!("Fur vectors unimplemented, returning empty data");
        Ok(Self {})
    }
}
