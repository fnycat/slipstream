use std::sync::Arc;

use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::node::node::IrNodeType;

/// A filter that only shows certain objects/files in the dialog.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectFilter {
    /// Displays only files of the given type.
    Type(IrNodeType)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogResponse<T> {
    Open,
    Confirmed(T),
    Closed
}

pub struct ObjectDialog {
    arena: Arc<IrArena>,
    filter: ObjectFilter
}

impl ObjectDialog {
    pub fn new(filter: ObjectFilter, arena: Arc<IrArena>) -> Self {
        tracing::debug!("Opening object dialog with filter {filter:?}");
        Self { filter, arena }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) -> DialogResponse<IrNodeKey> {
        let Some(egui::InnerResponse { inner, .. }) = egui::Window::new("Object dialog")
            .collapsible(false)
            .show(ui, |ui| {
                if ui.button("Close").clicked() {
                    return DialogResponse::Closed;
                }

                DialogResponse::Open
            })
        else {
            return DialogResponse::Closed;
        };

        inner.expect("object dialog should not be collapsible")
    }
}
