use crate::{format::mdl0::tex_links::TextureLinks, node::node::Inspectable};

impl Inspectable for TextureLinks {
    fn draw_properties(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("{self:#?}"));
    }
}
