use crate::{format::mdl0::objects::Object, r#virtual::node::Inspectable};

impl Inspectable for Object {
    fn draw_properties(&mut self, _editor: &mut crate::editor::Editor, ui: &mut egui::Ui) {
        ui.label(format!("{self:#?}"));
    }
}
