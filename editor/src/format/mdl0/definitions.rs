use crate::{
    format::{error::EncodingResult, mdl0::mdl0::SectionDeserialize},
    shared::util::RefCursor,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Definitions {}

impl SectionDeserialize for Definitions {
    fn deserialize_section(
        reader: &mut RefCursor<[u8]>,
        _header_start: u32,
    ) -> EncodingResult<Self> {
        tracing::error!("TODO: draw lists");
        Ok(Self {})
    }
}
