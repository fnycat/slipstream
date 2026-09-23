use crate::editor::Editor;
use crate::format::mdl0::bytecode::Bytecode;
use crate::node::node::Inspectable;

impl Inspectable for Bytecode {
    fn draw_properties(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("{self:?}"));

        for _ in 0..100 {
            ui.label("many contents");
        }
    }
}
