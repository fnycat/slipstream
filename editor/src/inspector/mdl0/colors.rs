use crate::{format::mdl0::colors::Colors, node::node::Inspectable};

impl Inspectable for Colors {
    fn draw_properties(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("{self:?}"));
    }
}
