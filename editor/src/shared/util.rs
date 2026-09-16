pub fn draw_vec_drag_values<T: egui::emath::Numeric, const N: usize>(
    mut input_field_size: egui::Vec2,
    labels: [&str; N],
    values: &mut [T; N],
    ui: &mut egui::Ui,
) {
    input_field_size.x /= N as f32;

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        for i in (0..N).rev() {
            ui.add_sized(input_field_size, egui::DragValue::new(&mut values[i]));
            if labels[i].is_empty() {
                let label_width = ui
                    .painter()
                    .layout_no_wrap(
                        "X:".to_owned(),
                        egui::FontId::default(),
                        egui::Color32::TRANSPARENT,
                    )
                    .rect
                    .width();
                ui.allocate_space(egui::vec2(label_width, input_field_size.y));
            } else {
                ui.label(labels[i]);
            }
        }
    });
}

pub fn draw_inspector_section_header(name: String, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let total_width = ui.available_width();
        let text_width = ui
            .painter()
            .layout_no_wrap(
                name.clone(),
                egui::FontId::default(),
                ui.visuals().text_color(),
            )
            .rect
            .width();

        let padding = 16.0;
        let line_width = ((total_width - text_width - padding) / 2.0).max(0.0);
        let separator_size = egui::vec2(line_width, ui.available_height());

        ui.add_space(0.01 * ui.available_height());
        ui.add_sized(separator_size, egui::Separator::default().horizontal());
        ui.label(name);
        ui.add_sized(separator_size, egui::Separator::default().horizontal());
        ui.add_space(0.01 * ui.available_height());
    });
}
