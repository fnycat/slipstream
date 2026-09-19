use crate::editor::Editor;
use crate::format::mdl0::bytecode::DrawList;
use crate::r#virtual::node::Inspectable;

impl Inspectable for DrawList {
    fn draw_properties(&mut self, _editor: &mut Editor, ui: &mut egui::Ui) {
        ui.label(format!("{self:?}"));
    }
}
