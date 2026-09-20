use crate::{format::mdl0::uvs::Uvs, r#virtual::node::Inspectable};

impl Inspectable for Uvs {
    fn draw_properties(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("{self:?}"));
    }
}
