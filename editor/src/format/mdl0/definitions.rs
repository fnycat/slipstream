use crate::{
    format::mdl0::SectionDeserialize,
    shared::util::RefCursor,
};
use crate::error::EditorResult;

#[derive(Debug, Clone, PartialEq)]
pub struct Definitions {}

impl SectionDeserialize for Definitions {
    fn deserialize_section(reader: &mut RefCursor<[u8]>, _header_start: u32) -> EditorResult<Self> {
        tracing::error!("TODO: draw lists");
        Ok(Self {})
    }
}
