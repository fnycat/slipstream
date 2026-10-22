use crate::SlipstreamResult;

pub struct HeaderIcons {
    pub open: egui::RichText,
    pub closed: egui::RichText
}

pub enum DropdownType {
    Regular,
    MenuButton
}

pub struct CollapseDescriptor<'ctx, F> {
    state_id: egui::Id,
    icons: Option<HeaderIcons>,
    label: egui::RichText,
    header_alignment: DropdownType,
    default_open: bool,
    indent: bool,

    on_body: F,
    // using dynamic dispatch so we don't have to deal with rustc not being able to infer
    // the generic parameter when the user passes `None`.
    on_ctx_menu: Option<&'ctx mut dyn FnMut(&mut egui::Ui)>
}

// Set `C` to a default function pointer so we don't have to explicitly name the type.
impl<'ctx, B> CollapseDescriptor<'ctx, B>
where
    B: FnMut(&mut egui::Ui) -> SlipstreamResult<()>
{
    pub fn new(
        state_id: egui::Id,
        label: egui::RichText,
        icons: Option<HeaderIcons>,
        header_alignment: DropdownType,
        default_open: bool,
        indent: bool,
        on_body: B,
        on_ctx_menu: Option<&'ctx mut dyn FnMut(&mut egui::Ui)>
    ) -> Self {
        Self {
            state_id,
            label,
            icons,
            header_alignment,
            default_open,
            indent,
            on_body,
            on_ctx_menu
        }
    }
}

/// Draws the icon of files and folders in the outliner.
pub fn draw_node_icon(ui: &mut egui::Ui, icon: egui::RichText, response: &egui::Response) {
    let galley = egui::WidgetText::from(icon).into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        f32::INFINITY,
        egui::TextStyle::Body
    );

    let center_pos = response.rect.center() - (galley.size() * 0.5);

    ui.painter()
        .galley(center_pos, galley, ui.visuals().text_color());
}

pub fn draw_node_selector<Label, Icon, Click>(
    id: impl egui::AsIdSalt,
    label: Label,
    ty: DropdownType,
    draw_icon: Icon,
    on_click: Click,
    // using dynamic dispatch so we don't have to deal with rustc not being able to infer
    // the generic parameter when the user passes `None`.
    mut on_ctx: Option<&mut dyn FnMut(&mut egui::Ui)>,
    ui: &mut egui::Ui
) -> egui::Response
where
    Label: Into<egui::WidgetText>,
    Icon: FnOnce(&mut egui::Ui) -> egui::Response,
    Click: FnOnce(&mut egui::Ui)
{
    // Determines the rect that should be coloured when the node is hovered over.
    let clip_rect = ui.clip_rect();
    // Need to slightly decrease the width due to possible file tree indentation.
    let row_width = clip_rect.width() - ui.cursor().min.x;
    let row_height = ui.spacing().interact_size.y;

    let header_rect = egui::Rect::from_min_size(
        ui.cursor().min.max(egui::Pos2::ZERO),
        egui::vec2(row_width, row_height)
    );

    assert!(
        !header_rect.any_nan(),
        "Collapsing header row rect had NaN entries: {header_rect:?}"
    );

    let row_response = ui.interact(header_rect, ui.id().with(id), egui::Sense::click());

    // If the cursor hovers over the node, fill the background with a different colour.
    if ui.rect_contains_pointer(header_rect) {
        ui.painter().rect_filled(
            header_rect,
            ui.visuals().widgets.hovered.corner_radius,
            ui.visuals().widgets.hovered.bg_fill
        );
    }

    // Draw the folder icon and label.
    ui.horizontal(|ui| {
        let icon_response = draw_icon(ui);

        let header_label = egui::Label::new(label).selectable(false);
        let label_response = match ty {
            DropdownType::Regular => ui.add(header_label),
            DropdownType::MenuButton => {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add(header_label)
                })
                .response
            }
        };

        // This is a hack, but the collapsing states responses kind of suck.
        //
        // We generate our own responses on the outliner row and label of the file, as the collapsing header does
        // not respond to these by default.
        // We also need to ensure the icon is not below the cursor, as the icon lies within the outliner row. Otherwise
        // the collapsing state itself will also respond and we will attempt to toggle the node twice.
        if (row_response.clicked() || label_response.clicked()) && !icon_response.hovered() {
            // state.toggle(ui);
            on_click(&mut *ui);
        }

        // We also need separate context menus for the row and label responses, although they both display the same content.
        if let Some(ctx_fn) = &mut on_ctx {
            row_response.context_menu(|ui| ctx_fn(ui));
            label_response.context_menu(|ui| ctx_fn(ui));
            icon_response.context_menu(|ui| ctx_fn(ui));
        }
    });

    if ui.rect_contains_pointer(header_rect) {
        // Set a custom cursor to make the outliner feel more responsive.
        //
        // This is done after drawing to override the select cursor for the label.
        ui.set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    row_response
}

pub fn draw_collapsing_state<F>(
    mut desc: CollapseDescriptor<F>,
    ui: &mut egui::Ui
) -> SlipstreamResult<()>
where
    F: FnMut(&mut egui::Ui) -> SlipstreamResult<()>
{
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        desc.state_id,
        desc.default_open
    );

    // need to set the type explicitly due to poor inference.
    let draw_icon = |ui: &mut egui::Ui| {
        if let Some(icon) = desc.icons {
            state.show_toggle_button(ui, move |ui, openness, response| {
                if openness > 0.5 {
                    draw_node_icon(ui, icon.open, response)
                } else {
                    draw_node_icon(ui, icon.closed, response)
                }
            })
        } else {
            state.show_toggle_button(ui, move |ui, openness, response| {
                egui::collapsing_header::paint_default_icon(ui, openness, response)
            })
        }
    };

    // need to set the type explicitly due to poor inference.
    let mut should_toggle = false;
    let on_click = |_ui: &mut egui::Ui| {
        // can't borrow `state` twice.
        should_toggle = true;
    };

    let header_response = draw_node_selector(
        desc.state_id,
        desc.label,
        desc.header_alignment,
        draw_icon,
        on_click,
        desc.on_ctx_menu,
        ui
    );

    if should_toggle {
        state.toggle(ui);
    }

    let body_response = if desc.indent {
        state.show_body_indented(&header_response, ui, |ui| (desc.on_body)(ui))
    } else {
        state.show_body_unindented(ui, |ui| (desc.on_body)(ui))
    };

    if let Some(egui::InnerResponse { inner, .. }) = body_response {
        inner?;
    }

    Ok(())
}
