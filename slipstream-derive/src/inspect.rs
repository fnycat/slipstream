// AI disclaimer: Most of this was written by me, but some modifications have been made by Claude
// because I spent way too much time trying to get the layout to be precisely what I wanted.
//
// Claude replaced the old grid-based layout by a manual one and also slightly improved the collapsing
// logic. All code has been double checked by me. - fnycat.

use darling::ast::{Data, Fields, Style};
use darling::{FromDeriveInput, FromMeta};
use heck::ToTitleCase;
use proc_macro2::Span;
use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use syn::{Expr, ExprLit, Ident, Lit, LitStr, Type};

trait StringAttrExt {
    fn to_tokens_opt(&self) -> proc_macro2::TokenStream;
}

impl StringAttrExt for Option<StringAttr> {
    fn to_tokens_opt(&self) -> proc_macro2::TokenStream {
        if let Some(attr) = self {
            quote! { Some(#attr) }
        } else {
            quote! { None  }
        }
    }
}

#[derive(Debug)]
struct StringAttr {
    expr: Expr
}

impl FromMeta for StringAttr {
    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        match value {
            syn::Lit::Str(s) => Ok(Self {
                expr: Expr::Lit(ExprLit {
                    attrs: Vec::new(),
                    lit: Lit::Str(s.clone())
                })
            }),
            _ => Err(darling::Error::unexpected_lit_type(value))
        }
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self {
            expr: Expr::Lit(ExprLit {
                attrs: Vec::new(),
                lit: Lit::Str(LitStr::new(value, Span::call_site()))
            })
        })
    }

    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        Ok(Self { expr: expr.clone() })
    }
}

impl ToTokens for StringAttr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let content = &self.expr;
        tokens.append_all(quote! { #content });
    }
}

#[inline]
const fn default_opened() -> bool {
    true
}

#[inline]
const fn default_max_horizontal_elements() -> usize {
    4
}

#[derive(Debug, darling::FromField)]
#[darling(attributes(inspect))]
struct FieldOpt {
    pub ident: Option<Ident>,
    pub ty: Type,

    #[darling(default = default_max_horizontal_elements)]
    pub max_horizontal_elements: usize,
    /// A tooltip to display when hovering over the value.
    #[darling(default)]
    pub tooltip: Option<StringAttr>,
    /// A prefix to put in front of the value. This is used in draggable values.
    #[darling(default)]
    pub prefix: Option<StringAttr>,
    /// A suffix to put behind the value. This is used in draggable values to put a degrees circle
    /// behind rotations for example.
    #[darling(default)]
    pub suffix: Option<StringAttr>,
    /// The lower bound of this value.
    #[darling(default)]
    pub min: Option<syn::Expr>,
    /// The upper bound of this value.
    #[darling(default)]
    pub max: Option<syn::Expr>,
    /// Whether this value is read only and cannot be edited.
    #[darling(default)]
    pub read_only: bool,
    /// The category to put this value in.
    #[darling(default)]
    pub category: Option<StringAttr>,
    /// Whether to ignore this field.
    ///
    /// It will not be shown in the property window and the value also does not have
    /// to implement the `Inspect` trait.
    #[darling(default)]
    pub ignore: bool,
    /// Gives the field a different name.
    ///
    /// By default, the field's name in the struct is converted to Title Case and used.
    /// But this can be used to override the name.
    #[darling(default)]
    pub rename: Option<String>,
    /// Nested structs start out expanded: `#[inspect(opened)]`.
    #[darling(default = "default_opened")]
    pub opened: bool,
    /// Draws this field with the given function.
    #[darling(default)]
    pub with: Option<syn::Path>
}

impl FieldOpt {
    /// An expression that evaluates to the value to inspect. The caller takes `&mut` of it.
    pub fn accessor(&self, is_bitfield: bool) -> proc_macro2::TokenStream {
        match &self.ident {
            Some(x) => {
                if is_bitfield {
                    let set_fn = format_ident!("set_{x}");
                    quote! { slipstream_shared::inspect::BitFieldWrapper { value: self.#x(), on_update: |v| self.#set_fn(v) } }
                } else {
                    quote! { self.#x }
                }
            }
            None => todo!("tuple structs are not supported yet")
        }
    }

    pub fn name(&self) -> String {
        match (&self.rename, &self.ident) {
            (Some(rename), _) => rename.clone(),
            (None, Some(ident)) => ident.to_string().to_title_case(),
            (None, None) => String::from("<unknown>")
        }
    }

    pub fn config(&self) -> proc_macro2::TokenStream {
        let Self {
            ident,
            ty,
            read_only,
            min,
            max,
            ..
        } = self;

        let read_only = *read_only;
        let range = match (min, max) {
            (None, None) => quote! {
                // explicit type annotations are required here due to both nones.
                None
            },
            (Some(min), Some(max)) => {
                quote! {
                    {
                        use slipstream_shared::inspect::IntoBounds;

                        // verify that this type support bounds.
                        const _: () = {
                            const fn assert_impl<T: ?Sized + IntoBounds<#ty>>() {}
                            let _ = assert_impl::<#ty>();
                        };
                        // #ty is specified twice because we need to both specify the generic and the impl we want to use.
                        Some(<#ty as IntoBounds::<#ty>>::into_bounds(Some(#min), Some(#max)))
                    }
                }
            }
            (Some(min), None) => quote! {
                {
                    use slipstream_shared::inspect::IntoBounds;

                    // verify that this type support bounds.
                    const _: () = {
                        const fn assert_impl<T: ?Sized + IntoBounds<#ty>>() {}
                        let _ = assert_impl::<#ty>();
                    };
                    // #ty is specified twice because we need to both specify the generic and the impl we want to use.
                    Some(<#ty as IntoBounds::<#ty>>::into_bounds(Some(#min), None))
                }
            },
            (None, Some(max)) => quote! {
                {
                    use slipstream_shared::inspect::IntoBounds;

                    // verify that this type support bounds.
                    const _: () = {
                        const fn assert_impl<T: ?Sized + IntoBounds<#ty>>() {}
                        let _ = assert_impl::<#ty>();
                    };
                    // #ty is specified twice because we need to both specify the generic and the impl we want to use.
                    Some(<#ty as IntoBounds::<#ty>>::into_bounds(None, Some(#max)))
                }
            }
        };

        let label = if let Some(name) = &self.rename {
            quote! {
                #name
            }
        } else {
            quote! {
                stringify!(#ident)
            }
        };

        let category = self.category.to_tokens_opt();
        let prefix = self.prefix.to_tokens_opt();
        let suffix = self.suffix.to_tokens_opt();
        let tooltip = self.tooltip.to_tokens_opt();
        let max_horiz_elements = self.max_horizontal_elements;

        quote! {
            slipstream_shared::inspect::FieldConfig {
                label: #label,
                tooltip: #tooltip,
                category: #category,
                read_only: #read_only,
                range: #range,
                prefix: #prefix,
                suffix: #suffix,
                max_horizontal_elements: #max_horiz_elements
            }
        }
    }

    pub fn to_tokens(
        parent: &Ident,
        fields: &[FieldOpt],
        is_bitfield: bool
    ) -> proc_macro2::TokenStream {
        let fields = fields.iter().filter(|f| {
            // skip over field when it has the ignore attribute, or when it has a _ prefix in bitfield structs.
            let bitfield_ignore = if is_bitfield {
                f.ident
                    .as_ref()
                    .map(|n| n.to_string().starts_with("_"))
                    .unwrap_or(false)
            } else {
                false
            };

            !f.ignore && !bitfield_ignore
        });

        let mut tokens = proc_macro2::TokenStream::new();
        for field in fields {
            let name = field.name();
            let accessor = field.accessor(is_bitfield);
            let config = field.config();
            let default_open = field.opened;

            if let Some(draw_fn) = &field.with {
                tokens.append_all(quote! {
                    {
                        const CONFIG: slipstream_shared::inspect::FieldConfig = #config;
                        let value = &mut #accessor;
                        let field_id = id.with(#name);

                        changes |= widgets::property::leaf(ui, field_id, #name, |ui| {
                            #draw_fn(&mut *value, ui, &CONFIG)
                        });
                    }
                });
            } else {
                tokens.append_all(quote! {
                    {
                        const CONFIG: slipstream_shared::inspect::FieldConfig = #config;
                        let value = &mut #accessor;
                        let field_id = id.with(#name);

                        if slipstream_shared::inspect::Inspect::is_nested(&*value) {
                            // Nested struct: clickable header row plus an animated, indented body.
                            let summary = slipstream_shared::inspect::Inspect::summary(&*value);
                            let nested_changes = widgets::property::nested(
                                ui,
                                field_id,
                                #name,
                                summary.as_deref().unwrap_or(""),
                                #default_open,
                                |ui| slipstream_shared::inspect::Inspect::draw_rows(&mut *value, ui, field_id),
                            );

                            // `None` while the body is fully closed.
                            if let Some(nested_changes) = nested_changes {
                                changes |= nested_changes;
                            }
                        } else {
                            changes |= widgets::property::leaf(ui, field_id, #name, |ui| {
                                slipstream_shared::inspect::Inspect::draw_inner(&mut *value, ui, &CONFIG)
                            });
                        }
                    }
                });
            }
        }

        tokens
    }
}

#[derive(Debug, darling::FromVariant)]
#[darling(attributes(inspect))]
struct FieldVariant {
    pub ident: Ident,
    pub fields: Fields<FieldOpt>,

    #[darling(default)]
    pub tooltip: Option<String>,
    #[darling(default)]
    pub rename: Option<String>,
    #[darling(default)]
    pub ignore: bool
}

impl FieldVariant {
    pub fn name(&self) -> String {
        if let Some(rename) = &self.rename {
            rename.clone()
        } else {
            self.ident.to_string().to_title_case()
        }
    }

    pub fn to_tokens(names: &[String], variants: &[FieldVariant]) -> proc_macro2::TokenStream {
        let variants = variants
            .iter()
            .enumerate()
            .filter(|(_, f)| f.ignore == false);

        let mut tokens = proc_macro2::TokenStream::new();
        for (i, variant) in variants {
            let name = &names[i];
            let ident = &variant.ident;

            let tooltip = match &variant.tooltip {
                Some(tooltip) => quote! { Some(#tooltip) },
                // we have to set the type explicitly here because the compiler is unable to infer it.
                None => quote! { None::<&'static str> }
            };

            tokens.append_all(quote! {
                let is_selected = slipstream_shared::inspect::AsEnumLabel::as_index(self) == #i;
                let mut response = ui.selectable_label(is_selected, #name);

                if let Some(tooltip) = #tooltip {
                    response = response.on_hover_text(tooltip);
                }

                if response.clicked() {
                    *self = Self::#ident;
                    changes.changed = true;
                }
            });
        }

        tokens
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(inspect))]
pub struct Input {
    pub ident: Ident,
    pub data: darling::ast::Data<FieldVariant, FieldOpt>,

    #[darling(default)]
    pub rename: Option<String>,
    /// Optional `fn(&Self) -> String` used for the text shown next to a collapsed struct:
    /// `#[inspect(summary = "my_summary_fn")]`. Defaults to the struct's display name.
    #[darling(default)]
    pub summary: Option<syn::Path>
}

impl Input {
    fn name(&self) -> String {
        if let Some(rename) = &self.rename {
            rename.clone()
        } else {
            self.ident.to_string().to_title_case()
        }
    }

    fn struct_to_tokens(
        &self,
        fields: &[FieldOpt],
        is_bitfield: bool,
        tokens: &mut proc_macro2::TokenStream
    ) {
        let Self { ident, .. } = self;

        let name = self.name();
        let fields = FieldOpt::to_tokens(ident, fields, is_bitfield);

        let summary = match &self.summary {
            Some(path) => quote! { #path(self) },
            None => quote! { ::std::string::String::from(#name) }
        };

        tokens.append_all(quote! {
            /// Automatically generated by the [`Inspect`] derive macro.
            ///
            /// This function generates an abstract representation of the current struct
            #[cfg(feature = "egui-impls")]
            impl slipstream_shared::inspect::Inspect for #ident {
                /// Root of the property window: a column of rows that at least fills the panel.
                /// Nested structs are drawn as animated collapsing bodies by `draw_rows`.
                fn draw_properties(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
                    let mut changes = slipstream_shared::inspect::Changes::default();
                    let root_id = ui.id().with(concat!("InspectRoot_", #name));

                    ui.scope(|ui| {
                        // Gives the right aligned value cells a definite right edge.
                        ui.set_min_width(ui.available_width());
                        changes |= slipstream_shared::inspect::Inspect::draw_rows(self, ui, root_id);
                    });

                    changes
                }

                /// Fallback for callers that draw a whole struct as a single value.
                fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
                    let mut changes = slipstream_shared::inspect::Changes::default();
                    let id = ui.id().with(concat!("InspectNested_", #name));

                    ui.vertical(|ui| {
                        changes |= slipstream_shared::inspect::Inspect::draw_rows(self, ui, id);
                    });

                    changes
                }

                /// Draws one row per field into `ui`. `id` is unique to this struct instance and
                /// is the base for the ids of its fields.
                fn draw_rows(&mut self, ui: &mut egui::Ui, id: egui::Id) -> slipstream_shared::inspect::Changes {
                    use slipstream_shared::widgets;

                    let mut changes = slipstream_shared::inspect::Changes::default();
                    #fields
                    changes
                }

                fn is_nested(&self) -> bool {
                    true
                }

                fn summary(&self) -> Option<String> {
                    Some(#summary)
                }
            }
        });
    }

    fn enum_to_tokens(&self, variants: &[FieldVariant], tokens: &mut proc_macro2::TokenStream) {
        let Self { ident, .. } = self;

        let names = variants.iter().map(|v| v.name()).collect::<Vec<_>>();
        let fields = FieldVariant::to_tokens(&names, variants);

        let variant_names = variants.iter().zip(names.iter()).map(|(variant, name)| {
            let ident = &variant.ident;

            let pattern_content = match variant.fields.style {
                Style::Struct => Some(quote! { { .. } }),
                Style::Tuple => Some(quote! { (..) }),
                Style::Unit => None
            };

            let pattern = quote! {
                Self::#ident #pattern_content
            };

            quote! {
                #pattern => #name,
            }
        });

        let variant_indices = variants.iter().enumerate().map(|(i, variant)| {
            let ident = &variant.ident;

            let field_count = variant.fields.fields.len();
            let pattern_content = match variant.fields.style {
                Style::Struct => Some(quote! { { .. } }),
                Style::Tuple => {
                    // Repeat the `_` pattern to match the whole tuple.
                    let dash_repeat = std::iter::repeat(quote! { _ }).take(field_count);
                    Some(quote! {
                        (#(#dash_repeat),*)
                    })
                }
                Style::Unit => None
            };

            let pattern = quote! {
                Self::#ident #pattern_content
            };

            quote! {
                #pattern => #i,
            }
        });

        tokens.append_all(quote! {
            impl slipstream_shared::inspect::AsEnumLabel for #ident {
                #[inline]
                fn as_index(&self) -> usize {
                    match self {
                        #(#variant_indices)*
                    }
                }

                #[inline]
                fn as_label(&self) -> &'static str {
                    match self {
                        #(#variant_names)*
                    }
                }
            }

            // Enums are leaf rows: a combo box in the value cell. They don't override
            // `is_nested`, `summary` or `draw_rows`.
            #[cfg(feature = "egui-impls")]
            impl slipstream_shared::inspect::Inspect for #ident {
                fn draw_properties(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
                    todo!("draw enum as root (should be added in `Inspect` proc macro)");
                }

                fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
                    let curr_label = slipstream_shared::inspect::AsEnumLabel::as_label(self);

                    let mut changes = slipstream_shared::inspect::Changes::default();
                    let response = egui::ComboBox::new(ui.id().with("ComboBox"), "")
                        .selected_text(curr_label)
                        .show_ui(ui, |ui| {
                            #fields
                        }).response;

                    if let Some(tooltip) = cfg.tooltip {
                        response.on_hover_text(tooltip);
                    }

                    changes
                }
            }
        })
    }

    pub fn into_token_stream(self, is_bitfield: bool) -> proc_macro2::TokenStream {
        let mut tokens = proc_macro2::TokenStream::new();

        match &self.data {
            Data::Struct(data) => self.struct_to_tokens(&data.fields, is_bitfield, &mut tokens),
            Data::Enum(data) => self.enum_to_tokens(&data, &mut tokens)
        }

        tokens
    }
}
