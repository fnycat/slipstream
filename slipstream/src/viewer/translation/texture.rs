use slipstream_ir::node::arena::IrArena;
use slipstream_shared::SlipstreamResult;

use crate::viewer::translation::{IntermediateModel, ModelContents};

impl ModelContents<'_> {
    pub fn translate_textures(&self, model: &mut IntermediateModel, arena: &IrArena) -> SlipstreamResult<()> {
        Ok(())
    }
}