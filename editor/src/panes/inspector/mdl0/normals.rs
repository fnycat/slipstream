use crate::editor::Editor;
use crate::format::mdl0::normals::NormalBuf;
use crate::node::node::Inspectable;

impl Inspectable for NormalBuf {
    fn draw_properties(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Format: {:?}", self.format));
        ui.label(format!("Divisor: {}", self.divisor));
        ui.label(format!("Stride: {}", self.stride));
        ui.label(format!("Normal count: {}", self.normals.len()));
    }
}
