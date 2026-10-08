use slipstream_shared::SlipstreamResult;

pub struct HeaderIcons {
    pub open: egui::RichText,
    pub closed: egui::RichText,
}

pub struct CollapseDescriptor<B, C> {
    pub state_id: egui::Id,
    pub icons: Option<HeaderIcons>,
    pub label: egui::RichText,

    pub on_body: B,
    pub on_context_menu: C,
}

/// Draws the icon of files and folders in the outliner.
pub fn draw_header_icon(ui: &mut egui::Ui, icon: egui::RichText, response: &egui::Response) {
    let galley = egui::WidgetText::from(icon).into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        f32::INFINITY,
        egui::TextStyle::Body,
    );

    let center_pos = response.rect.center() - (galley.size() * 0.5);

    ui.painter()
        .galley(center_pos, galley, ui.visuals().text_color());
}

pub fn draw_collapsing_state<B, C>(
    desc: CollapseDescriptor<B, C>,
    ui: &mut egui::Ui,
) -> SlipstreamResult<()>
where
    B: FnOnce(&mut egui::Ui) -> SlipstreamResult<()>,
    C: Fn(&mut egui::Ui),
{
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        desc.state_id,
        false,
    );

    // Determines the rect that should be coloured when the node is hovered over.
    let row_height = ui.spacing().interact_size.y;
    let row_rect = egui::Rect::from_min_size(
        ui.cursor().min,
        egui::vec2(ui.available_width(), row_height),
    );

    let row_response = ui.interact(
        row_rect,
        desc.state_id.with("interact"),
        egui::Sense::click(),
    );

    // If the cursor hovers over the node, fill the background with a different colour.
    if ui.rect_contains_pointer(row_rect) {
        ui.painter().rect_filled(
            row_rect,
            ui.visuals().widgets.hovered.corner_radius,
            ui.visuals().widgets.hovered.bg_fill,
        );
    }

    ui.horizontal(|ui| {
        // Draw the folder icon and label.
        //
        // This block also handles responses.
        ui.allocate_ui(egui::vec2(row_height, row_height), |ui| {
            let icon_response = desc.icons.map(|icon| {
                state.show_toggle_button(ui, move |ui, openness, response| {
                    if openness > 0.5 {
                        draw_header_icon(ui, icon.open, response)
                    } else {
                        draw_header_icon(ui, icon.closed, response)
                    }
                })
            });

            let label_response = ui.label(desc.label);

            // This is a hack, but the collapsing states responses kind of suck.
            //
            // We generate our own responses on the outliner row and label of the file, as the collapsing header does
            // not respond to these by default.
            // We also need to ensure the icon is not below the cursor, as the icon lies within the outliner row. Otherwise
            // the collapsing state itself will also respond and we will attempt to toggle the node twice.
            if (row_response.clicked() || label_response.clicked())
                && !icon_response.map(|r| r.hovered()).unwrap_or(false)
            {
                state.toggle(ui);
            }

            // We also need separate context menus for the row and label responses, although they both display the same content.
            row_response.context_menu(|ui| (desc.on_context_menu)(ui));
            label_response.context_menu(|ui| (desc.on_context_menu)(ui));
        });

        if ui.rect_contains_pointer(row_rect) {
            // Set a custom cursor to make the outliner feel more responsive.
            ui.set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    });

    let body_response = state.show_body_indented(&row_response, ui, |ui| (desc.on_body)(ui));

    if let Some(egui::InnerResponse { inner, .. }) = body_response {
        inner?;
    }

    Ok(())
}
