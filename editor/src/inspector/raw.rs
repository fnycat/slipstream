use crate::editor::Editor;
use crate::shared::util::RefCursor;
use crate::r#virtual::node::Inspectable;

#[derive(Debug, Clone, PartialEq)]
pub struct Raw {
    pub bytes: RefCursor<[u8]>,
}

impl Inspectable for Raw {
    fn draw_properties(&mut self, _editor: &mut Editor, ui: &mut egui::Ui) {
        ui.label(format!("File size: {} bytes", self.bytes.remaining_len()));
    }
}
