use std::ops::ControlFlow;
use std::sync::Arc;

use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::node::node::{IrNode, IrNodeType};

/// A filter that only shows certain objects/files in the dialog.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectFilter {
    /// Displays only files of the given type.
    Type(IrNodeType)
}

impl ObjectFilter {
    pub fn matches(&self, node: &IrNode) -> bool {
        match self {
            Self::Type(ty) => node.ty() == *ty
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogResponse<T> {
    Open,
    Confirmed(T),
    Closed
}

pub struct ObjectDialog {
    // Cache the results instead of recomputing them every frame.
    is_cached: bool,
    cache: Vec<(IrNodeKey, String)>,

    arena: Arc<IrArena>,
    filter: ObjectFilter
}

impl ObjectDialog {
    pub fn new(filter: ObjectFilter, arena: Arc<IrArena>) -> Self {
        tracing::debug!("Opening object dialog with filter {filter:?}");
        Self {
            filter,
            arena,

            is_cached: false,
            cache: Vec::new()
        }
    }

    fn update_cache(&mut self) {
        self.cache.clear();
        let _ = self.arena.walk_all(IrNodeKey::ROOT, |node| {
            if self.filter.matches(node) {
                self.cache.push((node.key(), node.label.clone()));
            }
            Ok(ControlFlow::Continue(()))
        });

        tracing::debug!("Cached {} object dialog entries", self.cache.len());
        self.is_cached = true;
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) -> DialogResponse<IrNodeKey> {
        if !self.is_cached {
            self.update_cache();
        }

        let Some(egui::InnerResponse { inner, .. }) = egui::Window::new("Object dialog")
            .collapsible(false)
            .show(ui, |ui| {
                if ui.button("Close").clicked() {
                    return DialogResponse::Closed;
                }

                for (key, label) in &self.cache {
                    if ui
                        .selectable_label(false, format!("{label} ({key:?})"))
                        .clicked()
                    {
                        return DialogResponse::Confirmed(*key);
                    }
                }

                DialogResponse::Open
            })
        else {
            return DialogResponse::Closed;
        };

        inner.expect("object dialog should not be collapsible")
    }
}
