use crate::editor::Editor;
use crate::format::mdl0::normals::Normals;
use crate::r#virtual::node::Inspectable;

impl Inspectable for Normals {
    fn draw_properties(&mut self, _editor: &mut Editor, ui: &mut egui::Ui) {
        ui.label(format!("Format: {:?}", self.format));
        ui.label(format!("Divisor: {}", self.divisor));
        ui.label(format!("Stride: {}", self.stride));
        ui.label(format!("Normal count: {}", self.normals.len()));
    }
}
