use egui::Ui;
use crate::format::mdl0::tevs::Tev;
use crate::node::node::Inspectable;

impl Inspectable for Tev {
    fn draw_properties(&mut self, ui: &mut Ui) {
        ui.label(format!("{self:#?}"));
    }
}