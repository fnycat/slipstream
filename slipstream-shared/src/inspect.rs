use std::ops::{BitOr, BitOrAssign, Range, RangeBounds, RangeInclusive};

pub const DRAG_INPUT_SIZE: egui::Vec2 = egui::vec2(70.0, 20.0);

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
    /// The category to put the value in.
    pub category: Option<&'static str>,
    /// Whether the value cannot be edited.
    pub read_only: bool,
    /// The range of a slider.
    pub range: Option<RangeInclusive<f64>>,
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
            suffix: None,
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
    fn draw_inner(&mut self, ui: &mut egui::Ui, _cfg: &FieldConfig) -> Changes {
        let checkbox = egui::Checkbox::new(self, "");
        ui.add(checkbox).into()
    }
}

impl Inspect for i32 {
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        let drag_value = egui::DragValue::new(self);
        let drag_value = if let Some(range) = &cfg.range {
            drag_value.range(range.clone())
        } else {
            drag_value
        };
        ui.add_sized(DRAG_INPUT_SIZE, drag_value).into()
    }
}

impl Inspect for u32 {
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        let drag_value = egui::DragValue::new(self);
        let drag_value = if let Some(range) = &cfg.range {
            drag_value.range(range.clone())
        } else {
            drag_value
        };
        ui.add_sized(DRAG_INPUT_SIZE, drag_value).into()
    }
}

macro_rules! impl_vector {
    ($ty:ty, $($ident:ident),*) => {
        impl Inspect for $ty {
            #[inline]
            fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
                let mut response = Changes::default();
                $(
                    let mut drag_value = egui::DragValue::new(&mut self.$ident).speed(0.1);
                    if let Some(range) = &cfg.range {
                        drag_value = drag_value.range(range.clone());
                    }

                    if let Some(suffix) = cfg.suffix {
                        drag_value = drag_value.suffix(suffix);
                    }

                    response |= ui.add_sized(DRAG_INPUT_SIZE, drag_value).into();
                )*
                response
            }
        }
    }
}

impl_vector!(glam::Vec2, x, y);
impl_vector!(glam::Vec3, x, y, z);
impl_vector!(glam::Vec4, x, y, z, w);

impl Inspect for f32 {
    #[inline]
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        let drag_value = egui::DragValue::new(self);
        let drag_value = if let Some(range) = &cfg.range {
            drag_value.range(range.clone())
        } else {
            drag_value
        };
        ui.add_sized(DRAG_INPUT_SIZE, drag_value).into()
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
        self.iter_mut().fold(Changes::default(), |acc, val| {
            acc | val.draw_inner(ui, cfg).into()
        })
    }
}

impl<T: Inspect> Inspect for Vec<T> {
    fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &FieldConfig) -> Changes {
        self.iter_mut().fold(Changes::default(), |acc, val| {
            acc | val.draw_inner(ui, cfg).into()
        })
    }
}
