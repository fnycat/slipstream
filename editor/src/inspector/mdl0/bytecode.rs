use crate::editor::Editor;
use crate::format::mdl0::bytecode::Bytecode;
use crate::r#virtual::node::Inspectable;

impl Inspectable for Bytecode {
    fn draw_properties(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("{self:?}"));
    }
}
