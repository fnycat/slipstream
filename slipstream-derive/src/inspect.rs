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
pub struct StringAttr {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ContainerType {
    /// A regular struct.
    Struct,
    /// A bitfield.
    Bitfield
}

#[derive(Debug, darling::FromField)]
#[darling(attributes(inspect))]
pub struct FieldOpt {
    pub ident: Option<Ident>,
    pub ty: Type,

    /// Default initializer for this field. If not set, the `Default` trait is used instead.
    #[darling(default)]
    pub default: Option<syn::Path>,
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
    pub fn accessor(&self, ty: ContainerType) -> proc_macro2::TokenStream {
        if let Some(ident) = &self.ident {
            match ty {
                ContainerType::Bitfield => {
                    let set_fn = format_ident!("set_{ident}");
                    quote! { slipstream_shared::inspect::BitFieldWrapper { value: self.#ident(), on_update: |v| self.#set_fn(v) } }
                }
                ContainerType::Struct => {
                    quote! { self.#ident }
                }
            }
        } else {
            todo!("tuple structs are not supported yet");
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

    pub fn render(&self, ty: ContainerType, tokens: &mut proc_macro2::TokenStream) {
        let name = self.name();
        let accessor = self.accessor(ty);
        let config = self.config();
        let default_open = self.opened;

        if let Some(draw_fn) = &self.with {
            tokens.append_all(quote! {
                {
                    use slipstream_shared::widgets;

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
                    use slipstream_shared::widgets;

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
                            |ui| slipstream_shared::inspect::Inspect::draw_rows(&mut *value, ui, field_id, &CONFIG),
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

    pub fn render_all(fields: &[FieldOpt], ty: ContainerType) -> proc_macro2::TokenStream {
        let fields = fields.iter().filter(|f| {
            // skip over field when it has the ignore attribute, or when it has a _ prefix in bitfield structs.
            let bitfield_ignore = if ty == ContainerType::Bitfield {
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
            field.render(ty, &mut tokens);
        }

        tokens
    }
}

#[derive(Debug, darling::FromVariant)]
#[darling(attributes(inspect))]
pub struct FieldVariant {
    pub ident: Ident,
    pub fields: Fields<FieldOpt>,

    /// Function that returns a default value of the specified variant.
    #[darling(default)]
    pub default: Option<syn::Path>,
    /// A tooltip that is displayed when hovering over the variant.
    #[darling(default)]
    pub tooltip: Option<String>,
    /// Changes the displayed name of the variant.
    #[darling(default)]
    pub rename: Option<String>,
    /// Ignores the variant. It will not be shown to the user.
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

    /// Renders the variants in the dropdown menu.
    pub fn render(names: &[String], variants: &[FieldVariant]) -> proc_macro2::TokenStream {
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

            let setter = match variant.fields.style {
                darling::ast::Style::Unit => quote! { *self = Self::#ident },
                darling::ast::Style::Tuple => {
                    let inits = variant
                        .fields
                        .fields
                        .iter()
                        .map(|field| {
                            if let Some(default_fn) = &field.default {
                                quote! {
                                    #default_fn()
                                }
                            } else {
                                quote! {
                                    Default::default()
                                }
                            }
                        })
                        .collect::<Vec<_>>();

                    quote! {
                        *self = Self::#ident(#(#inits),*)
                    }
                }
                darling::ast::Style::Struct => {
                    // If there is a default function, use it.
                    // Otherwise attempt to default initialize all fields
                    // manually.

                    let inits = variant
                        .fields
                        .fields
                        .iter()
                        .map(|field| {
                            let ident = field
                                .ident
                                .as_ref()
                                .expect("struct field should have an identifier");

                            if let Some(default_fn) = &field.default {
                                quote! {
                                    #ident: #default_fn()
                                }
                            } else {
                                quote! {
                                    #ident: Default::default()
                                }
                            }
                        })
                        .collect::<Vec<_>>();

                    quote! {
                        *self = Self::#ident {
                            #(#inits),*
                        }
                    }
                }
            };

            tokens.append_all(quote! {
                let is_selected = slipstream_shared::inspect::AsEnumLabel::as_index(self) == #i;
                let mut response = ui.selectable_label(is_selected, #name);

                if let Some(tooltip) = #tooltip {
                    response = response.on_hover_text(tooltip);
                }

                if response.clicked() {
                    #setter;
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

    fn render_struct(
        &self,
        fields: &[FieldOpt],
        ty: ContainerType,
        tokens: &mut proc_macro2::TokenStream
    ) {
        let Self { ident, .. } = self;

        let name = self.name();
        let fields = FieldOpt::render_all(fields, ty);

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
                        changes |= slipstream_shared::inspect::Inspect::draw_rows(self, ui, root_id, cfg);
                    });

                    changes
                }

                /// Fallback for callers that draw a whole struct as a single value.
                fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
                    let mut changes = slipstream_shared::inspect::Changes::default();
                    let id = ui.id().with(concat!("InspectNested_", #name));

                    ui.vertical(|ui| {
                        changes |= slipstream_shared::inspect::Inspect::draw_rows(self, ui, id, cfg);
                    });

                    changes
                }

                /// Draws one row per field into `ui`. `id` is unique to this struct instance and
                /// is the base for the ids of its fields.
                fn draw_rows(&mut self, ui: &mut egui::Ui, id: egui::Id, _cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
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

    /// Generates a match statement over every variant in the enum, adding the tokens
    /// that the closure produces.
    fn match_on_variants<F>(
        &self,
        variants: &[FieldVariant],
        eval_fn: F
    ) -> Vec<proc_macro2::TokenStream>
    where
        F: Fn(usize, &FieldVariant) -> proc_macro2::TokenStream
    {
        variants
            .iter()
            .enumerate()
            .map(|(i, variant)| {
                let ident = &variant.ident;

                let pattern_content = match variant.fields.style {
                    Style::Struct => Some(quote! { { .. } }),
                    Style::Tuple => Some(quote! { (..) }),
                    Style::Unit => None
                };

                let pattern = quote! {
                    Self::#ident #pattern_content
                };

                let eval = eval_fn(i, variant);
                quote! {
                    #pattern => #eval,
                }
            })
            .collect()
    }

    fn render_enum(&self, variants: &[FieldVariant], tokens: &mut proc_macro2::TokenStream) {
        let Self { ident, .. } = self;

        let names = variants.iter().map(|v| v.name()).collect::<Vec<_>>();
        let fields = FieldVariant::render(&names, variants);

        // Generate match statement to return the enum variant name.
        let variant_names = self.match_on_variants(variants, |_, variant: &FieldVariant| {
            let name = variant.name();
            quote! { #name }
        });

        // Generate match statement to return the enum variant index.
        let variant_indices = self.match_on_variants(variants, |i, _| quote! { #i });

        // let enum_content = self.render_enum_content(variants);
        tokens.append_all(quote! {
            impl slipstream_shared::inspect::AsEnumLabel for #ident {
                fn as_index(&self) -> usize {
                    match self {
                        #(#variant_indices)*
                    }
                }

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
                    let err_text = egui::RichText::new(format!("ERROR: `Inspect::draw_properties` is not implemented for enum `{}`", stringify!(#ident)))
                        .color(ui.visuals().error_fg_color);

                    ui.label(err_text);

                    slipstream_shared::inspect::Changes::default()
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

                    // If the enum has content, display it below the ComboBox.
                    // #enum_content

                    changes
                }
            }
        })
    }

    pub fn into_token_stream(self, ty: ContainerType) -> proc_macro2::TokenStream {
        let mut tokens = proc_macro2::TokenStream::new();

        match &self.data {
            Data::Struct(data) => self.render_struct(&data.fields, ty, &mut tokens),
            Data::Enum(data) => self.render_enum(&data, &mut tokens)
        }

        tokens
    }
}
