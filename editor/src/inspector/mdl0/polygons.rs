use crate::{format::mdl0::polygons::Polygon, r#virtual::node::Inspectable};

impl Inspectable for Polygon {
    fn draw_properties(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("{self:#?}"));
    }
}
