use crate::{format::mdl0::pal_links::PaletteLinks, r#virtual::node::Inspectable};

impl Inspectable for PaletteLinks {
    fn draw_properties(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("{self:#?}"));
    }
}
