use egui::Ui;
use crate::format::mdl0::textures::Texture;
use crate::node::node::Inspectable;

impl Inspectable for Texture {
    fn draw_properties(&mut self, ui: &mut Ui) {
        ui.label(format!("{self:?}"));
    }
}