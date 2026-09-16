use crate::{
    format::mdl0::{BillboardSetting, BoneFlags, Bones},
    shared::{util::vec_drag_value, r#virtual::Inspectable},
};

impl Inspectable for BoneFlags {
    fn draw_properties(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(
            &mut self.apply_child_scale_compensate,
            "Enable child scale compensate",
        );

        ui.checkbox(&mut self.apply_scale_compensate, "Enable scale compensate");
        ui.end_row();

        ui.checkbox(&mut self.disable_classic_scale, "Disable classic scale");

        ui.checkbox(&mut self.is_billboard_child, "Billboard child").on_hover_text("UNCONFIRMED: Indicates that this bone's parent is a billboard, i.e. always facing the camera");
        ui.end_row();

        ui.checkbox(&mut self.is_display_matrix, "Display matrix");

        ui.checkbox(&mut self.is_visible, "Visible")
            .on_hover_text("UNCONFIRMED: Whether the geometry attached to this bone is visible");
        ui.end_row();

        ui.checkbox(
            &mut self.translation_isotropic,
            "Enable istropic translation",
        );

        ui.checkbox(&mut self.rotation_isotropic, "Enable isotropic rotation");
        ui.end_row();

        ui.checkbox(&mut self.scale_isotropic, "Enable isotropic scaling");

        ui.checkbox(&mut self.scale_uniform, "Enable uniform scaling")
            .on_hover_text("UNCONFIRMED: Optimization flag set when all scaling factors are equal");
        ui.end_row();

        ui.checkbox(&mut self.use_identity, "Use identity").on_hover_text("UNCONFIRMED: Fast-path for rendering, forces the engine to use the identity matrix for this bone (i.e. no translation, rotation or scale)");
        ui.end_row();
    }
}

impl Inspectable for Bones {
    fn draw_properties(&mut self, ui: &mut egui::Ui) {
        let input_field_size = egui::vec2(180.0, 20.0);

        // ui.columns_const(|[col1, col2]| {
        egui::Grid::new("properties_grid")
            .striped(true)
            .num_columns(2)
            .spacing([40.0, 8.0])
            .show(ui, |ui| {
                ui.label("Translation:");
                vec_drag_value(
                    input_field_size,
                    ["X:", "Y:", "Z:"],
                    &mut self.translation_vector,
                    ui,
                );

                ui.end_row();

                ui.label("Rotation:");
                vec_drag_value(
                    input_field_size,
                    ["X:", "Y:", "Z:"],
                    &mut self.rotation_vector,
                    ui,
                );

                ui.end_row();

                ui.label("Scale:");
                vec_drag_value(
                    input_field_size,
                    ["X:", "Y:", "Z:"],
                    &mut self.scaling_vector,
                    ui,
                );

                ui.end_row();

                ui.label("Bounding volume minimum:");
                vec_drag_value(
                    input_field_size,
                    ["X:", "Y:", "Z:"],
                    &mut self.bounding_volume_min,
                    ui,
                );

                ui.end_row();

                ui.label("Bounding volume maximum:");
                vec_drag_value(
                    input_field_size,
                    ["X:", "Y:", "Z:"],
                    &mut self.bounding_volume_max,
                    ui,
                );
                ui.end_row();

                let separator = egui::Separator::default().horizontal();

                ui.add(separator);

                ui.end_row();

                ui.label("Bone index:");
                ui.add(egui::DragValue::new(&mut self.index));

                ui.end_row();

                ui.label("Billboard setting:");
                ui.menu_button(format!("{:?}", self.billboard_setting), |ui| {
                    for i in 0..BillboardSetting::len() {
                        let setting = BillboardSetting::try_from(i as u32)
                            .expect("billboard setting size is outdated");

                        if ui.button(format!("{setting:?}")).clicked() {
                            self.billboard_setting = setting;
                        }
                    }
                });

                ui.end_row();

                self.flags.draw_properties(ui);
            });
    }
}
