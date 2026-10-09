use crate::{SlipstreamResult, error::AssertFailed, verify};

pub struct HeaderIcons {
    pub open: egui::RichText,
    pub closed: egui::RichText,
}

pub enum HeaderAlignment {
    Left,
    Right,
}

pub struct CollapseDescriptor<B, C> {
    state_id: egui::Id,
    icons: Option<HeaderIcons>,
    label: egui::RichText,
    header_alignment: HeaderAlignment,
    default_open: bool,
    indent: bool,

    on_body: B,
    on_ctx_menu: Option<C>,
}

// Set `C` to a default function pointer so we don't have to explicitly name the type.
impl<B> CollapseDescriptor<B, fn(&mut egui::Ui)>
where
    B: FnMut(&mut egui::Ui) -> SlipstreamResult<()>,
{
    pub fn new(
        state_id: egui::Id,
        label: egui::RichText,
        icons: Option<HeaderIcons>,
        header_alignment: HeaderAlignment,
        default_open: bool,
        indent: bool,
        on_body: B,
    ) -> Self {
        Self {
            state_id,
            label,
            icons,
            header_alignment,
            default_open,
            indent,
            on_body,
            on_ctx_menu: None,
        }
    }
}

impl<B, C> CollapseDescriptor<B, C>
where
    B: FnMut(&mut egui::Ui) -> SlipstreamResult<()>,
    C: FnMut(&mut egui::Ui),
{
    pub fn with_context_menu(
        state_id: egui::Id,
        label: egui::RichText,
        icons: Option<HeaderIcons>,
        header_alignment: HeaderAlignment,
        default_open: bool,
        indent: bool,
        on_body: B,
        on_ctx_menu: C,
    ) -> Self {
        Self {
            state_id,
            label,
            icons,
            header_alignment,
            default_open,
            indent,
            on_body,
            on_ctx_menu: Some(on_ctx_menu),
        }
    }
}

/// Draws the icon of files and folders in the outliner.
pub fn draw_header_icon<'a, 'r>(
    ui: &'a mut egui::Ui,
    icon: egui::RichText,
    response: &'r egui::Response,
) {
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

pub fn draw_collapsing_state<'a, 'b, B, C>(
    mut desc: CollapseDescriptor<B, C>,
    ui: &'a mut egui::Ui,
) -> SlipstreamResult<()>
where
    B: FnMut(&mut egui::Ui) -> SlipstreamResult<()>,
    C: FnMut(&mut egui::Ui),
{
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        desc.state_id,
        desc.default_open,
    );

    // Determines the rect that should be coloured when the node is hovered over.
    let clip_rect = ui.clip_rect();
    // Need to slightly decrease the width due to possible file tree indentation.
    let row_width = clip_rect.width() - ui.cursor().min.x;
    let row_height = ui.spacing().interact_size.y;

    let header_rect = egui::Rect::from_min_size(
        ui.cursor().min.max(egui::Pos2::ZERO),
        egui::vec2(row_width, row_height),
    );

    assert!(
        !header_rect.any_nan(),
        "Collapsing header row rect had NaN entries: {header_rect:?}"
    );

    let row_response = ui.interact(
        header_rect,
        desc.state_id.with("interact"),
        egui::Sense::click(),
    );

    // If the cursor hovers over the node, fill the background with a different colour.
    if ui.rect_contains_pointer(header_rect) {
        ui.painter().rect_filled(
            header_rect,
            ui.visuals().widgets.hovered.corner_radius,
            ui.visuals().widgets.hovered.bg_fill,
        );
    }

    // Draw the folder icon and label.
    ui.horizontal(|ui| {
        let icon_response = if let Some(icon) = desc.icons {
            state.show_toggle_button(ui, move |ui, openness, response| {
                if openness > 0.5 {
                    draw_header_icon(ui, icon.open, response)
                } else {
                    draw_header_icon(ui, icon.closed, response)
                }
            })
        } else {
            state.show_toggle_button(ui, move |ui, openness, response| {
                egui::collapsing_header::paint_default_icon(ui, openness, response)
            })
        };

        let header_label = egui::Label::new(desc.label).selectable(false);
        let label_response = match desc.header_alignment {
            HeaderAlignment::Left => ui.add(header_label),
            HeaderAlignment::Right => {
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
            state.toggle(ui);
        }

        // We also need separate context menus for the row and label responses, although they both display the same content.
        if let Some(ctx_fn) = &mut desc.on_ctx_menu {
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

    let body_response = if desc.indent {
        state.show_body_indented(&row_response, ui, |ui| (desc.on_body)(ui))
    } else {
        state.show_body_unindented(ui, |ui| (desc.on_body)(ui))
    };

    if let Some(egui::InnerResponse { inner, .. }) = body_response {
        inner?;
    }

    Ok(())
}
