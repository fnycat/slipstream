use std::ops::ControlFlow;

use crate::chr;
use crate::visitor::{VisitorCtx, VisitorCtxMut};

pub trait Chr0Visitor {
    /// Whether to stop traversing the tree when an unimplemented visitor method is encountered.
    fn stop_when_uninterested(&self) -> bool {
        false
    }

    // CHR visitor methods
    // ==================================================================================================

    fn visit_chr(
        &mut self,
        animations: VisitorCtx<'_, chr::CharacterAnimation>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_animation(
        &mut self,
        animation: VisitorCtx<'_, chr::SkeletalAnimation>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_chr_mut(
        &mut self,
        animation: VisitorCtxMut<'_, chr::CharacterAnimation>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_animation_mut(
        &mut self,
        animation: VisitorCtxMut<'_, chr::SkeletalAnimation>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}
