use crate::inspect::{Changes, FieldConfig};

/// Right aligned value area that fills the rest of the row. The height is bounded, so vertical
/// centering can never make the row grow inside a `ScrollArea`. `id` gives the cell a unique
/// `ui.id()`, which widgets like `ComboBox` derive their own ids from.
pub fn value_cell<R>(ui: &mut egui::Ui, id: egui::Id, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let rect = egui::Rect::from_min_size(
        ui.cursor().min,
        egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
    );

    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt(id)
            .max_rect(rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        add,
    )
    .inner
}

/// A regular (non nested) row: label on the left, value on the right.
pub fn leaf<R>(
    ui: &mut egui::Ui,
    id: egui::Id,
    label: &str,
    value: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    ui.horizontal(|ui| {
        ui.add(
            egui::Label::new(format!("{label}: "))
                .wrap_mode(egui::TextWrapMode::Extend)
                .selectable(false),
        );
        value_cell(ui, id.with("value"), value)
    })
    .inner
}

/// A nested struct: a header row with an arrow, the label and a weak summary, followed by an
/// animated, indented body. The whole header row is clickable and gets a hover highlight.
///
/// Returns `None` while the body is fully closed, otherwise whatever `body` returned.
pub fn nested<R>(
    ui: &mut egui::Ui,
    id: egui::Id,
    label: &str,
    summary: &str,
    default_open: bool,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        id,
        default_open,
    );

    // Reserve a spot below the text for the hover highlight.
    let hover_bg = ui.painter().add(egui::Shape::Noop);
    let gap = ui.spacing().item_spacing.y * 0.5;

    let header = ui.horizontal(|ui| {
        let openness = state.openness(ui.ctx());
        let icon_size = ui.spacing().icon_width * 0.7;
        let (_, icon) = ui.allocate_exact_size(egui::Vec2::splat(icon_size), egui::Sense::hover());

        egui::collapsing_header::paint_default_icon(ui, openness, &icon);

        ui.add(
            egui::Label::new(format!("{label}: "))
                .wrap_mode(egui::TextWrapMode::Extend)
                .selectable(false),
        );

        // A weak summary instead of a second arrow, so it reads as a hint, not a button.
        value_cell(ui, id.with("summary"), |ui| {
            ui.add(
                egui::Label::new(egui::RichText::new(summary).weak())
                    .selectable(false)
                    .wrap_mode(egui::TextWrapMode::Truncate),
            );
        });
    });

    // One click target over the whole row.
    let rect = header.response.rect;
    let header_resp = ui.interact(rect, id.with("header"), egui::Sense::click());

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

    if header_resp.clicked() {
        state.toggle(ui);
    }

    // Clips the body to `openness * height` while animating, and stores the state.
    state
        .show_body_indented(&header_resp, ui, body)
        .map(|response| response.inner)
}

/// Draws the array by wrapping every `per_row` entries.
///
/// This is used to for example draw the polygon UV Array IDs in two 4-entry rows instead of a
/// large one of 8 entries.
pub fn wrapped<T>(
    ui: &mut egui::Ui,
    id: egui::Id,
    items: &mut [T],
    per_row: usize,
    mut add: impl FnMut(&mut egui::Ui, &mut T),
) {
    ui.vertical(|ui| {
        for (row, chunk) in items.chunks_mut(per_row.max(1)).enumerate() {
            value_cell(ui, id.with(row), |ui| {
                for item in chunk.iter_mut().rev() {
                    add(ui, item);
                }
            })
        }
    });
}

pub const DRAG_INPUT_SIZE: egui::Vec2 = egui::vec2(70.0, 20.0);

pub fn drag_value(
    mut drag_value: egui::DragValue<'_>,
    cfg: &FieldConfig,
    ui: &mut egui::Ui,
) -> Changes {
    if let Some(range) = &cfg.range {
        drag_value = drag_value.range(range.clone());
    }

    if let Some(prefix) = cfg.prefix {
        drag_value = drag_value.prefix(prefix);
    }

    if let Some(suffix) = cfg.suffix {
        drag_value = drag_value.suffix(suffix);
    }

    let mut response = ui.add_sized(DRAG_INPUT_SIZE, drag_value);
    if let Some(tooltip) = cfg.tooltip {
        response = response.on_hover_text(tooltip);
    }
    response.into()
}
