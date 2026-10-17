use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::mdl::DeserializeContents;
use crate::node::node::IrNodeType;

pub struct UserData {}

impl DeserializeContents for UserData {
    const KIND: IrNodeType = IrNodeType::UserData;
    const NAME: &str = "User Data";

    fn deserialize_contents(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        tracing::warn!("MDL user data unimplemented, returning empty data");
        Ok(Self {})
    }
}
