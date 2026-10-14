use std::ops::ControlFlow;

use crate::chr0;
use crate::visitor::{VisitorContext, VisitorContextMut};

pub trait Chr0Visitor {
    /// Whether to stop traversing the tree when an unimplemented visitor method is encountered.
    fn stop_when_uninterested(&self) -> bool {
        false
    }

    // CHR0 visitor methods
    // ==================================================================================================

    fn visit_chr0(&mut self, animations: VisitorContext<'_, chr0::Chr0Root>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_animation(
        &mut self,
        animation: VisitorContext<'_, chr0::SkeletalAnimation>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_chr0_mut(
        &mut self,
        animation: VisitorContextMut<'_, chr0::Chr0Root>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_animation_mut(
        &mut self,
        animation: VisitorContextMut<'_, chr0::SkeletalAnimation>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}
