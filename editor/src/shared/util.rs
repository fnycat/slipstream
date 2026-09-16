pub fn vec_drag_value<T: egui::emath::Numeric, const N: usize>(
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
