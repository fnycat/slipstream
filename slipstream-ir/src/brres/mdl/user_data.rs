use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::brres::{DeserializeContents, MdlSectionType};
use crate::node::node::IrNodeType;

pub struct UserData {}

impl DeserializeContents for UserData {
    const KIND: IrNodeType = IrNodeType::MdlSection {
        ty: MdlSectionType::UserData
    };
    const NAME: &str = "User Data";

    fn deserialize_contents(_reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        tracing::warn!("MDL user data unimplemented, returning empty data");
        Ok(Self {})
    }
}
