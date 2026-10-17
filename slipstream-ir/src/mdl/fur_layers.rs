use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::mdl::DeserializeContents;
use crate::node::node::IrNodeType;

pub struct FurLayers {}

impl DeserializeContents for FurLayers {
    const KIND: IrNodeType = IrNodeType::FurLayers;
    const NAME: &str = "Fur Layers";

    fn deserialize_contents(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        tracing::warn!("Fur layers unimplemented, returning empty data");
        Ok(Self {})
    }
}
