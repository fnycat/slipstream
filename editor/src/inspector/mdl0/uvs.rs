use crate::{format::mdl0::uvs::UvBuf, node::node::Inspectable};

impl Inspectable for UvBuf {
    fn draw_properties(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("{self:?}"));
    }
}
