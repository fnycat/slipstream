use std::ops::{BitOr, BitOrAssign, Range, RangeBounds, RangeInclusive};

use egui::emath;

use crate::widgets::{self, drag_value};

/// The maximum amount of input entries that will be allowed on a single line.
/// This is used for large arrays such as the UV array IDs, which would otherwise display
/// 8 input fields next to each other.
pub const MAX_HORIZONTAL_INPUT_COUNT: usize = 3;

#[diagnostic::on_unimplemented(
    label = "non-numerical type",
    message = "The attributes `min` and `max` cannot be used on this field's type"
)]
pub trait IntoBounds<T: Into<f64>> {
    fn into_bounds(min: Option<T>, max: Option<T>) -> RangeInclusive<f64>;
}

pub trait AsEnumLabel {
    /// Converts the current enum variant to its index in the variant list.
    /// This may not correspond to the actual discriminant!
    fn as_index(&self) -> usize;
    /// Converts the current enum variant to a human-readable name.
    fn as_label(&self) -> &'static str;
}

/// Describes changes made by the window
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Changes {
    pub changed: bool,
}

impl From<Option<egui::Response>> for Changes {
    fn from(value: Option<egui::Response>) -> Self {
        Self {
            changed: value.map(|c| c.changed()).unwrap_or(false),
        }
    }
}

impl From<egui::Response> for Changes {
    fn from(value: egui::Response) -> Self {
        Self {
            changed: value.changed(),
        }
    }
}

impl BitOr for Changes {
    type Output = Changes;

    fn bitor(self, rhs: Self) -> Self::Output {
        Changes {
            changed: self.changed || rhs.changed,
        }
    }
}

impl BitOrAssign for Changes {
    fn bitor_assign(&mut self, rhs: Self) {
        self.changed |= rhs.changed
    }
}

pub trait Inspect {
    /// Draws the labels and values.
    ///
    /// This is called to draw the full properties window.
    #[inline]
    fn draw_properties(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        self.draw_inner(ui, cfg)
    }

    /// Draws only the value of the property.
    ///
    /// This is called on fields of structs.
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes;

    /// Draws one grid row per field into the grid of the given `ui`.
    fn draw_rows(&mut self, ui: &mut egui::Ui, id: egui::Id) -> Changes {
        Changes::default()
    }

    fn is_nested(&self) -> bool {
        false
    }

    /// Generates a summary of the nested struct's content.
    ///
    /// This is for example used for bounding volumes that have been collapsed.
    fn summary(&self) -> Option<String> {
        None
    }
}

#[derive(Debug)]
pub struct FieldConfig {
    pub label: &'static str,
    /// Shows a tooltip when hovering over a value.
    pub tooltip: Option<&'static str>,
    /// The category to put the value in.
    pub category: Option<&'static str>,
    /// Whether the value cannot be edited.
    pub read_only: bool,
    /// The range of a slider.
    pub range: Option<RangeInclusive<f64>>,
    pub prefix: Option<&'static str>,
    /// The suffix to add to the drag values.
    pub suffix: Option<&'static str>,
}

impl FieldConfig {
    pub const fn label(mut self, label: &'static str) -> Self {
        self.label = label;
        self
    }

    pub const fn category(mut self, category: &'static str) -> Self {
        self.category = Some(category);
        self
    }

    pub const fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn range<T: Into<f64> + Clone>(mut self, range: RangeInclusive<T>) -> Self {
        self.range = Some(range.start().clone().into()..=range.end().clone().into());
        self
    }
}

impl Default for FieldConfig {
    fn default() -> Self {
        Self {
            prefix: None,
            suffix: None,
            tooltip: None,
            label: "<unknown>",
            category: None,
            read_only: false,
            range: None,
        }
    }
}

pub struct BitFieldWrapper<T, F> {
    pub value: T,
    pub on_update: F,
}

impl<T: Inspect + Copy, F: FnMut(T)> Inspect for BitFieldWrapper<T, F> {
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        let response = self.value.draw_inner(ui, cfg).into();
        (self.on_update)(self.value);
        response
    }
}

impl Inspect for bool {
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        let checkbox = egui::Checkbox::new(self, "");
        let response = ui.add(checkbox);
        if let Some(tooltip) = cfg.tooltip {
            response.on_hover_text(tooltip)
        } else {
            response
        }
        .into()
    }
}

impl Inspect for u8 {
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        let val = egui::DragValue::new(self);
        drag_value(val, cfg, ui)
    }
}

impl Inspect for i16 {
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        let val = egui::DragValue::new(self);
        drag_value(val, cfg, ui)
    }
}

impl Inspect for u16 {
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        let val = egui::DragValue::new(self);
        drag_value(val, cfg, ui)
    }
}

impl Inspect for i32 {
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        let val = egui::DragValue::new(self);
        drag_value(val, cfg, ui)
    }
}

impl Inspect for u32 {
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        let val = egui::DragValue::new(self);
        drag_value(val, cfg, ui)
    }
}

impl Inspect for u64 {
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        let val = egui::DragValue::new(self);
        drag_value(val, cfg, ui)
    }
}

macro_rules! impl_vector {
    ($ty:ty, $($ident:ident),*) => {
        impl Inspect for $ty {
            #[inline]
            fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
                let mut response = Changes::default();
                $(
                    let val = egui::DragValue::new(&mut self.$ident).speed(0.1);
                    response |= drag_value(val, cfg, ui);
                )*
                response
            }
        }
    }
}

// Components need to be loaded in reverse due to the right to left layout.

impl_vector!(glam::Vec2, y, x);
impl_vector!(glam::Vec3, z, y, x);
impl_vector!(glam::Vec4, w, z, y, x);

impl_vector!(glam::U8Vec2, y, x);
impl_vector!(glam::U8Vec3, z, y, x);
impl_vector!(glam::U8Vec4, w, z, y, x);

impl Inspect for f32 {
    #[inline]
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        let val = egui::DragValue::new(self);
        drag_value(val, cfg, ui)
    }
}

impl<T: Inspect> Inspect for Option<T> {
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        match self {
            Some(x) => x.draw_inner(ui, cfg).into(),
            None => ui.label("None").into(),
        }
    }
}

impl<T: Inspect, const N: usize> Inspect for [T; N] {
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        let mut changes = Changes::default();
        let id = ui.id().with("Array");

        widgets::wrapped(ui, id, self, MAX_HORIZONTAL_INPUT_COUNT, |ui, item| {
            changes |= item.draw_inner(ui, cfg);
        });

        changes
    }
}

impl<T: Inspect> Inspect for Vec<T> {
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        let mut changes = Changes::default();
        let id = ui.id().with("Array");

        widgets::wrapped(ui, id, self, MAX_HORIZONTAL_INPUT_COUNT, |ui, item| {
            changes |= item.draw_inner(ui, cfg);
        });

        changes
    }
}
