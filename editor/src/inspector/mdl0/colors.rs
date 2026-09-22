use crate::{format::mdl0::colors::ColorBuf, node::node::Inspectable};

impl Inspectable for ColorBuf {
    fn draw_properties(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("{self:?}"));
    }
}
