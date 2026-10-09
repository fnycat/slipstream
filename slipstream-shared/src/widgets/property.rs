/// Vertical guide lines in the indentation area, one per ancestor level.
fn draw_guides(ui: &egui::Ui, left: f32, y: egui::Rangef, depth: usize) {
    let indent = ui.spacing().indent;
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;

    for level in 0..depth {
        let x = (left + level as f32 * indent + indent * 0.5).round();
        ui.painter().vline(x, y, stroke);
    }
}

/// Label cell of a regular (non nested) row.
pub fn leaf_label(ui: &mut egui::Ui, depth: usize, label: &str) {
    let indent = ui.spacing().indent * depth as f32;
    let gap = ui.spacing().item_spacing.y * 0.5;

    let row = ui.horizontal(|ui| {
        let left = ui.max_rect().left();
        ui.add_space(indent);
        ui.add(
            egui::Label::new(format!("{label}: "))
                .wrap_mode(egui::TextWrapMode::Extend)
                .selectable(false),
        );
        left
    });

    draw_guides(
        ui,
        row.inner,
        row.response.rect.y_range().expand(gap),
        depth,
    );
}

/// Value cell. Right aligned, with a bounded height so that vertical centering can never make
/// the row grow inside a `ScrollArea`.
pub fn value_cell<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
    ui.allocate_ui_with_layout(size, egui::Layout::right_to_left(egui::Align::Center), add)
        .inner
}

/// Draws the header row of a nested struct (label cell, value cell and `end_row`).
///
/// The arrow, the label and the summary all toggle the foldout, the label gets a hover
/// highlight, and the open state is stored per `id`. Returns whether the struct is open, in which
/// case the caller must draw the child rows right after this call.
pub fn nested_header(
    ui: &mut egui::Ui,
    id: egui::Id,
    depth: usize,
    label: &str,
    summary: &str,
    default_open: bool,
) -> f32 {
    let mut open = ui.data_mut(|d| *d.get_temp_mut_or(id, default_open));

    // Reserve a spot below the text for the hover highlight.
    let hover_bg = ui.painter().add(egui::Shape::Noop);

    let indent = ui.spacing().indent * depth as f32;
    let gap = ui.spacing().item_spacing.y * 0.5;

    let header = ui.horizontal(|ui| {
        let left = ui.max_rect().left();
        ui.add_space(indent);

        let openness = ui.ctx().animate_bool(id.with("openness"), open);
        let (_, icon) = ui.allocate_exact_size(
            egui::Vec2::splat(ui.spacing().icon_width),
            egui::Sense::hover(),
        );
        egui::collapsing_header::paint_default_icon(ui, openness, &icon);

        ui.add(
            egui::Label::new(format!("{label}: "))
                .wrap_mode(egui::TextWrapMode::Extend)
                .selectable(false),
        );
        left
    });

    let rect = header.response.rect;
    let header_resp = ui.interact(rect, id.with("header"), egui::Sense::click());
    let mut toggled = header_resp.clicked();

    if header_resp.hovered() {
        let visuals = &ui.visuals().widgets.hovered;
        ui.painter().set(
            hover_bg,
            egui::epaint::RectShape::filled(
                rect.expand2(egui::vec2(2.0, gap)),
                visuals.corner_radius,
                visuals.weak_bg_fill,
            ),
        );
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    draw_guides(ui, header.inner, rect.y_range().expand(gap), depth);

    // Value cell: a weak summary instead of a second arrow, so it reads as a hint, not a button.
    let summary_resp = value_cell(ui, |ui| {
        ui.add(
            egui::Label::new(egui::RichText::new(summary).weak())
                .selectable(false)
                .sense(egui::Sense::click()),
        )
    });
    if summary_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    toggled |= summary_resp.clicked();
    ui.end_row();

    if toggled {
        open = !open;
        ui.data_mut(|d| d.insert_temp(id, open));
    }

    // open
    ui.ctx().animate_bool(id.with("openness"), open)
}
