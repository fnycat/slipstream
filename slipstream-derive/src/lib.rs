// use darling::ast::{Data, Fields, Style};
// use darling::{FromDeriveInput, FromField};
// use heck::ToTitleCase;
// use proc_macro::TokenStream;
// use proc_macro2::Span;
// use quote::{ToTokens, TokenStreamExt, format_ident, quote};
// use syn::parse::{Parse, ParseStream};
// use syn::spanned::Spanned;
// use syn::{Attribute, Ident, Meta, Token, Type, Visibility};
// use syn::{PathSegment, Result};
//
// #[derive(Debug, darling::FromField)]
// #[darling(attributes(inspect))]
// struct FieldOpt {
//     pub ident: Option<Ident>,
//     pub ty: Type,
//
//     #[darling(default)]
//     pub tooltip: Option<String>,
//     #[darling(default)]
//     pub compact: bool,
//     #[darling(default)]
//     pub suffix: Option<String>,
//     #[darling(default)]
//     pub min: Option<syn::Expr>,
//     #[darling(default)]
//     pub max: Option<syn::Expr>,
//     #[darling(default)]
//     pub read_only: bool,
//     #[darling(default)]
//     pub category: Option<String>,
//     #[darling(default)]
//     pub ignore: bool,
//     #[darling(default)]
//     pub rename: Option<String>,
//     #[darling(default)]
//     pub with: Option<syn::Path>,
// }
//
// impl FieldOpt {
//     pub fn accessor(&self, is_bitfield: bool) -> proc_macro2::TokenStream {
//         match &self.ident {
//             Some(x) => {
//                 if is_bitfield {
//                     let set_fn = format_ident!("set_{x}");
//                     quote! { slipstream_shared::inspect::BitFieldWrapper { value: self.#x(), on_update: |v| self.#set_fn(v) } }
//                 } else {
//                     quote! { self.#x }
//                 }
//             }
//             None => todo!("tuple structs are not supported yet"),
//         }
//     }
//
//     pub fn name(&self) -> String {
//         match (&self.rename, &self.ident) {
//             (Some(rename), _) => rename.to_title_case(),
//             (None, Some(ident)) => ident.to_string().to_title_case(),
//             (None, None) => String::from("<unknown>"),
//         }
//     }
//
//     pub fn config(&self) -> proc_macro2::TokenStream {
//         let Self {
//             ident,
//             ty,
//             read_only,
//             min,
//             max,
//             suffix,
//             ..
//         } = self;
//
//         let read_only = *read_only;
//         let range = match (min, max) {
//             (None, None) => quote! {
//                 // explicit type annotations are required here due to both nones.
//                 None
//             },
//             (Some(min), Some(max)) => {
//                 quote! {
//                     {
//                         use slipstream_shared::inspect::IntoBounds;
//
//                         // verify that this type support bounds.
//                         const _: () = {
//                             const fn assert_impl<T: ?Sized + IntoBounds<#ty>>() {}
//                             let _ = assert_impl::<#ty>();
//                         };
//                         // #ty is specified twice because we need to both specify the generic and the impl we want to use.
//                         Some(<#ty as IntoBounds::<#ty>>::into_bounds(Some(#min), Some(#max)))
//                     }
//                 }
//             }
//             (Some(min), None) => quote! {
//                 {
//                     use slipstream_shared::inspect::IntoBounds;
//
//                     // verify that this type support bounds.
//                     const _: () = {
//                         const fn assert_impl<T: ?Sized + IntoBounds<#ty>>() {}
//                         let _ = assert_impl::<#ty>();
//                     };
//                     // #ty is specified twice because we need to both specify the generic and the impl we want to use.
//                     Some(<#ty as IntoBounds::<#ty>>::into_bounds(Some(#min), None))
//                 }
//             },
//             (None, Some(max)) => quote! {
//                 {
//                     use slipstream_shared::inspect::IntoBounds;
//
//                     // verify that this type support bounds.
//                     const _: () = {
//                         const fn assert_impl<T: ?Sized + IntoBounds<#ty>>() {}
//                         let _ = assert_impl::<#ty>();
//                     };
//                     // #ty is specified twice because we need to both specify the generic and the impl we want to use.
//                     Some(<#ty as IntoBounds::<#ty>>::into_bounds(None, Some(#max)))
//                 }
//             },
//         };
//
//         let label = if let Some(name) = &self.rename {
//             quote! {
//                 #name
//             }
//         } else {
//             quote! {
//                 stringify!(#ident)
//             }
//         };
//
//         let category = if let Some(category) = &self.category {
//             quote! {
//                 Some(#category)
//             }
//         } else {
//             quote! {
//                 None
//             }
//         };
//
//         let suffix = if let Some(suffix) = &self.suffix {
//             quote! {
//                 Some(#suffix)
//             }
//         } else {
//             quote! {
//                 None
//             }
//         };
//
//         quote! {
//             slipstream_shared::inspect::FieldConfig {
//                 label: #label,
//                 category: #category,
//                 read_only: #read_only,
//                 range: #range,
//                 suffix: #suffix
//             }
//         }
//     }
//
//     pub fn to_tokens(
//         parent: &Ident,
//         fields: &[FieldOpt],
//         is_bitfield: bool,
//     ) -> proc_macro2::TokenStream {
//         let fields = fields.iter().filter(|f| {
//             // skip over field when it has the ignore attribute, or when it has a _ prefix in bitfield structs.
//             let bitfield_ignore = if is_bitfield {
//                 f.ident
//                     .as_ref()
//                     .map(|n| n.to_string().starts_with("_"))
//                     .unwrap_or(false)
//             } else {
//                 false
//             };
//
//             !f.ignore && !bitfield_ignore
//         });
//
//         let mut tokens = proc_macro2::TokenStream::new();
//         for field in fields {
//             let name = field.name();
//             let accessor = field.accessor(is_bitfield);
//             let config = field.config();
//
//             tokens.append_all(quote! {
//                 ui.with_layout(
//                     egui::Layout::top_down(egui::Align::Min),
//                     |ui| {
//                         let label = egui::Label::new(concat!(#name, ": ")).wrap_mode(egui::TextWrapMode::Extend);
//                         ui.add(label);
//                     }
//                 );
//
//                 ui.with_layout(
//                     egui::Layout::right_to_left(egui::Align::Min),
//                     |ui| {
//                         const CONFIG: slipstream_shared::inspect::FieldConfig = #config;
//                         let response = #accessor.draw_inner(ui, &CONFIG);
//                         changes |= response;
//                     }
//                 );
//                 ui.end_row();
//             });
//         }
//
//         tokens
//     }
// }
//
// #[derive(Debug, darling::FromVariant)]
// #[darling(attributes(inspect))]
// struct FieldVariant {
//     pub ident: Ident,
//     pub fields: Fields<FieldOpt>,
//
//     #[darling(default)]
//     pub tooltip: Option<String>,
//     #[darling(default)]
//     pub rename: Option<String>,
//     #[darling(default)]
//     pub ignore: bool,
// }
//
// impl FieldVariant {
//     pub fn name(&self) -> String {
//         if let Some(rename) = &self.rename {
//             rename.clone()
//         } else {
//             self.ident.to_string().to_title_case()
//         }
//     }
//
//     pub fn to_tokens(names: &[String], variants: &[FieldVariant]) -> proc_macro2::TokenStream {
//         let variants = variants
//             .iter()
//             .enumerate()
//             .filter(|(_, f)| f.ignore == false);
//
//         let mut tokens = proc_macro2::TokenStream::new();
//         for (i, variant) in variants {
//             let name = &names[i];
//             let ident = &variant.ident;
//
//             tokens.append_all(quote! {
//                 let is_selected = slipstream_shared::inspect::AsEnumLabel::as_index(self) == #i;
//                 let response = ui.selectable_label(is_selected, #name);
//
//                 if response.clicked() {
//                     *self = Self::#ident;
//                     changes.changed = true;
//                 }
//             });
//         }
//
//         tokens
//     }
// }
//
// #[derive(Debug, FromDeriveInput)]
// #[darling(attributes(inspect))]
// struct Input {
//     pub ident: Ident,
//     pub data: darling::ast::Data<FieldVariant, FieldOpt>,
//
//     #[darling(default)]
//     pub rename: Option<String>,
// }
//
// impl Input {
//     fn name(&self) -> String {
//         if let Some(rename) = &self.rename {
//             rename.clone()
//         } else {
//             self.ident.to_string().to_title_case()
//         }
//     }
//
//     fn struct_to_tokens(
//         &self,
//         fields: &[FieldOpt],
//         is_bitfield: bool,
//         tokens: &mut proc_macro2::TokenStream,
//     ) {
//         let Self { ident, .. } = self;
//
//         let name = self.name();
//         let fields = FieldOpt::to_tokens(ident, fields, is_bitfield);
//
//         tokens.append_all(quote! {
//             /// Automatically generated by the [`Inspect`] derive macro.
//             ///
//             /// This function generates an abstract representation of the current struct
//             // #ty is specified twice because we need to both specify the generic and the impl we want to use.
//             impl slipstream_shared::inspect::Inspect for #ident {
//                 fn draw_properties(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
//                     let mut changes = slipstream_shared::inspect::Changes::default();
//                     egui::Grid::new(concat!("GridRoot_", #name))
//                         .num_columns(2)
//                         .show(ui, |ui| {
//                             #fields
//                         });
//
//                     changes
//                 }
//
//                 fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
//                     use slipstream_shared::widgets;
//
//                     let mut changes = slipstream_shared::inspect::Changes::default();
//                     ui.vertical(|ui| {
//                         let desc = widgets::CollapseDescriptor::new(
//                             ui.id().with(concat!("NestedCollapsingHeader_", #name)),
//                             #name.into(),
//                             None,
//                             widgets::DropdownType::MenuButton,
//                             false,
//                             false,
//                             |ui| {
//                                 egui::Grid::new(concat!("GridNested_", #name))
//                                     .num_columns(2)
//                                     .show(ui, |ui| {
//                                         #fields
//                                     });
//
//                                 Ok(())
//                             }
//                         );
//
//                         widgets::draw_collapsing_state(desc, ui).expect("failed to draw collapsing state in inspector");
//                     });
//
//                     changes
//                 }
//             }
//         });
//     }
//
//     fn enum_to_tokens(&self, variants: &[FieldVariant], tokens: &mut proc_macro2::TokenStream) {
//         let Self { ident, .. } = self;
//
//         let names = variants.iter().map(|v| v.name()).collect::<Vec<_>>();
//         let fields = FieldVariant::to_tokens(&names, variants);
//
//         let variant_names = variants.iter().zip(names.iter()).map(|(variant, name)| {
//             let ident = &variant.ident;
//
//             let field_count = variant.fields.fields.len();
//             let pattern_content = match variant.fields.style {
//                 Style::Struct => Some(quote! { { .. } }),
//                 Style::Tuple => {
//                     // Repeat the `_` pattern to match the whole tuple.
//                     let dash_repeat = std::iter::repeat(quote! { _ }).take(field_count);
//                     Some(quote! {
//                         (#(#dash_repeat),*)
//                     })
//                 }
//                 Style::Unit => None,
//             };
//
//             let pattern = quote! {
//                 Self::#ident #pattern_content
//             };
//
//             quote! {
//                 #pattern => #name,
//             }
//         });
//
//         let variant_indices = variants.iter().enumerate().map(|(i, variant)| {
//             let ident = &variant.ident;
//
//             let field_count = variant.fields.fields.len();
//             let pattern_content = match variant.fields.style {
//                 Style::Struct => Some(quote! { { .. } }),
//                 Style::Tuple => {
//                     // Repeat the `_` pattern to match the whole tuple.
//                     let dash_repeat = std::iter::repeat(quote! { _ }).take(field_count);
//                     Some(quote! {
//                         (#(#dash_repeat),*)
//                     })
//                 }
//                 Style::Unit => None,
//             };
//
//             let pattern = quote! {
//                 Self::#ident #pattern_content
//             };
//
//             quote! {
//                 #pattern => #i,
//             }
//         });
//
//         tokens.append_all(quote! {
//             impl slipstream_shared::inspect::AsEnumLabel for #ident {
//                 #[inline]
//                 fn as_index(&self) -> usize {
//                     match self {
//                         #(#variant_indices)*
//                     }
//                 }
//
//                 #[inline]
//                 fn as_label(&self) -> &'static str {
//                     match self {
//                         #(#variant_names)*
//                     }
//                 }
//             }
//
//             impl slipstream_shared::inspect::Inspect for #ident {
//                 fn draw_properties(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
//                     todo!();
//                 }
//
//                 fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
//                     let curr_label = slipstream_shared::inspect::AsEnumLabel::as_label(self);
//
//                     let mut changes = slipstream_shared::inspect::Changes::default();
//                     egui::ComboBox::new(ui.id().with("ComboBox"), "")
//                         .selected_text(curr_label)
//                         .show_ui(ui, |ui| {
//                             #fields
//                         });
//
//                     changes
//                 }
//             }
//         })
//     }
//
//     pub fn into_token_stream(self, is_bitfield: bool) -> proc_macro2::TokenStream {
//         let mut tokens = proc_macro2::TokenStream::new();
//
//         match &self.data {
//             Data::Struct(data) => self.struct_to_tokens(&data.fields, is_bitfield, &mut tokens),
//             Data::Enum(data) => self.enum_to_tokens(&data, &mut tokens),
//         }
//
//         tokens
//     }
// }
//
// /// Separate function because you cannot call proc macro functions directly.
// fn derive_inspect_inner(input: TokenStream, is_bitfield: bool) -> proc_macro::TokenStream {
//     let input = syn::parse_macro_input!(input as syn::DeriveInput);
//     let parsed = Input::from_derive_input(&input).expect("failed to parse input");
//     TokenStream::from(parsed.into_token_stream(is_bitfield))
// }
//
// /// Fields not marked with `pub` will not be shown in the property window by default.
// /// This can be overriden with `hidden = false`, the attribute will always take priority over field
// /// visibility.
// #[proc_macro_derive(Inspect, attributes(inspect))]
// pub fn derive_inspect(input: TokenStream) -> TokenStream {
//     derive_inspect_inner(input, false)
// }
//
// struct BitFieldType {
//     ty: Type,
// }
//
// impl Parse for BitFieldType {
//     fn parse(input: ParseStream) -> Result<Self> {
//         Ok(Self { ty: input.parse()? })
//     }
// }
//
// #[proc_macro_attribute]
// pub fn inspect_bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
//     let bitfield_ty = syn::parse_macro_input!(args as BitFieldType).ty;
//
//     // Need to convert proc_macro2 to proc_macro because `syn::parse_macro_input` requires it.
//     // But then `quote` requires proc_macro2 so we need to convert back again.
//     let inspect_derived = proc_macro2::TokenStream::from(derive_inspect_inner(input.clone(), true));
//     let input = proc_macro2::TokenStream::from(input);
//
//     let expanded = quote! {
//         #[bitfield(#bitfield_ty)]
//         #input
//         #inspect_derived
//     };
//
//     TokenStream::from(expanded)
// }

use darling::ast::{Data, Fields, Style};
use darling::{FromDeriveInput, FromField};
use heck::ToTitleCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Attribute, Ident, Meta, Token, Type, Visibility};
use syn::{PathSegment, Result};

#[derive(Debug, darling::FromField)]
#[darling(attributes(inspect))]
struct FieldOpt {
    pub ident: Option<Ident>,
    pub ty: Type,

    #[darling(default)]
    pub tooltip: Option<String>,
    #[darling(default)]
    pub compact: bool,
    #[darling(default)]
    pub suffix: Option<String>,
    #[darling(default)]
    pub min: Option<syn::Expr>,
    #[darling(default)]
    pub max: Option<syn::Expr>,
    #[darling(default)]
    pub read_only: bool,
    #[darling(default)]
    pub category: Option<String>,
    #[darling(default)]
    pub ignore: bool,
    /// Changes the displayed name of this field: `#[inspect(rename = "This is a new name")]`.
    #[darling(default)]
    pub rename: Option<String>,
    #[darling(default)]
    pub with: Option<syn::Path>,
    /// Nested structs start out expanded: `#[inspect(open)]`.
    #[darling(default)]
    pub open: bool,
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
            None => todo!("tuple structs are not supported yet"),
        }
    }

    pub fn name(&self) -> String {
        match (&self.rename, &self.ident) {
            (Some(rename), _) => rename.to_title_case(),
            (None, Some(ident)) => ident.to_string().to_title_case(),
            (None, None) => String::from("<unknown>"),
        }
    }

    pub fn config(&self) -> proc_macro2::TokenStream {
        let Self {
            ident,
            ty,
            read_only,
            min,
            max,
            suffix,
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
            },
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

        let category = if let Some(category) = &self.category {
            quote! {
                Some(#category)
            }
        } else {
            quote! {
                None
            }
        };

        let suffix = if let Some(suffix) = &self.suffix {
            quote! {
                Some(#suffix)
            }
        } else {
            quote! {
                None
            }
        };

        quote! {
            slipstream_shared::inspect::FieldConfig {
                label: #label,
                category: #category,
                read_only: #read_only,
                range: #range,
                suffix: #suffix
            }
        }
    }

    /// Generates the body of `Inspect::draw_rows`. It expects `ui`, `depth`, `id` and `changes`
    /// to be in scope, and `rows` to be the `inspect_rows` helper module.
    pub fn to_tokens(
        parent: &Ident,
        fields: &[FieldOpt],
        is_bitfield: bool,
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
            let default_open = field.open;

            // Each field gets its own block so the borrows of `self` don't overlap.
            tokens.append_all(quote! {
                {
                    const CONFIG: slipstream_shared::inspect::FieldConfig = #config;
                    let value = &mut #accessor;
                    let field_id = id.with(#name);

                    if slipstream_shared::inspect::Inspect::is_nested(&*value) {
                        // Nested struct: foldout header row, then the children in the same grid.
                        let summary = slipstream_shared::inspect::Inspect::summary(&*value);
                        let openness = widgets::property::nested_header(
                            ui,
                            field_id,
                            depth,
                            #name,
                            summary.as_deref().unwrap_or(""),
                            #default_open,
                        );

                        if openness > 0.0 {
                            // let prev_opacity = ui.opacity();
                            // ui.set_opacity(prev_opacity * openness);
                            ui.set_opacity(openness);

                            changes |= slipstream_shared::inspect::Inspect::draw_rows(
                                &mut *value,
                                ui,
                                depth + 1,
                                field_id,
                            );

                            // ui.set_opacity(prev_opacity);
                        }
                    } else {
                        widgets::property::leaf_label(ui, depth, #name);
                        changes |= widgets::property::value_cell(ui, |ui| {
                            slipstream_shared::inspect::Inspect::draw_inner(&mut *value, ui, &CONFIG)
                        });
                        ui.end_row();
                    }
                }
            });
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
    pub ignore: bool,
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

            tokens.append_all(quote! {
                let is_selected = slipstream_shared::inspect::AsEnumLabel::as_index(self) == #i;
                let response = ui.selectable_label(is_selected, #name);

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
struct Input {
    pub ident: Ident,
    pub data: darling::ast::Data<FieldVariant, FieldOpt>,

    #[darling(default)]
    pub rename: Option<String>,
    /// Optional `fn(&Self) -> String` used for the text shown next to a collapsed struct:
    /// `#[inspect(summary = "my_summary_fn")]`. Defaults to the struct's display name.
    #[darling(default)]
    pub summary: Option<syn::Path>,
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
        tokens: &mut proc_macro2::TokenStream,
    ) {
        let Self { ident, .. } = self;

        let name = self.name();
        let fields = FieldOpt::to_tokens(ident, fields, is_bitfield);

        let summary = match &self.summary {
            Some(path) => quote! { #path(self) },
            None => quote! { ::std::string::String::from(#name) },
        };

        tokens.append_all(quote! {
            /// Automatically generated by the [`Inspect`] derive macro.
            ///
            /// This function generates an abstract representation of the current struct
            impl slipstream_shared::inspect::Inspect for #ident {
                /// Root of the property window: one grid that at least fills the panel. Nested
                /// structs are drawn into this same grid by `draw_rows`.
                fn draw_properties(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
                    let mut changes = slipstream_shared::inspect::Changes::default();
                    let root_id = ui.id().with(concat!("InspectRoot_", #name));

                    ui.scope(|ui| {
                        // Gives the right aligned value cells a definite right edge.
                        ui.set_min_width(ui.available_width());

                        egui::Grid::new(root_id.with("Grid"))
                            .num_columns(2)
                            .show(ui, |ui| {
                                changes |= slipstream_shared::inspect::Inspect::draw_rows(self, ui, 0, root_id);
                            });
                    });

                    changes
                }

                /// Standalone fallback for places that cannot flatten into a parent grid.
                fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
                    let mut changes = slipstream_shared::inspect::Changes::default();
                    let id = ui.id().with(concat!("InspectNested_", #name));

                    ui.vertical(|ui| {
                        egui::Grid::new(id.with("Grid"))
                            .num_columns(2)
                            .show(ui, |ui| {
                                changes |= slipstream_shared::inspect::Inspect::draw_rows(self, ui, 0, id);
                            });
                    });

                    changes
                }

                /// Draws one grid row per field into the grid of the given `ui`.
                fn draw_rows(&mut self, ui: &mut egui::Ui, depth: usize, id: egui::Id) -> slipstream_shared::inspect::Changes {
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
                Style::Unit => None,
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
                Style::Unit => None,
            };

            let pattern = quote! {
                Self::#ident #pattern_content
            };

            quote! {
                #pattern => #i,
            }
        });

        let combobox_ident = format!("ComboBox_{}", ident.to_string());

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
            impl slipstream_shared::inspect::Inspect for #ident {
                fn draw_properties(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
                    todo!();
                }

                fn draw_inner(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
                    let curr_label = slipstream_shared::inspect::AsEnumLabel::as_label(self);

                    let mut changes = slipstream_shared::inspect::Changes::default();
                    egui::ComboBox::new(ui.id().with(#combobox_ident), "")
                        .selected_text(curr_label)
                        .show_ui(ui, |ui| {
                            #fields
                        });

                    changes
                }
            }
        })
    }

    pub fn into_token_stream(self, is_bitfield: bool) -> proc_macro2::TokenStream {
        let mut tokens = proc_macro2::TokenStream::new();

        match &self.data {
            Data::Struct(data) => self.struct_to_tokens(&data.fields, is_bitfield, &mut tokens),
            Data::Enum(data) => self.enum_to_tokens(&data, &mut tokens),
        }

        tokens
    }
}

/// Separate function because you cannot call proc macro functions directly.
fn derive_inspect_inner(input: TokenStream, is_bitfield: bool) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let parsed = Input::from_derive_input(&input).expect("failed to parse input");
    TokenStream::from(parsed.into_token_stream(is_bitfield))
}

/// Fields not marked with `pub` will not be shown in the property window by default.
/// This can be overriden with `hidden = false`, the attribute will always take priority over field
/// visibility.
#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn derive_inspect(input: TokenStream) -> TokenStream {
    derive_inspect_inner(input, false)
}

struct BitFieldType {
    ty: Type,
}

impl Parse for BitFieldType {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self { ty: input.parse()? })
    }
}

#[proc_macro_attribute]
pub fn inspect_bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    let bitfield_ty = syn::parse_macro_input!(args as BitFieldType).ty;

    // Need to convert proc_macro2 to proc_macro because `syn::parse_macro_input` requires it.
    // But then `quote` requires proc_macro2 so we need to convert back again.
    let inspect_derived = proc_macro2::TokenStream::from(derive_inspect_inner(input.clone(), true));
    let input = proc_macro2::TokenStream::from(input);

    let expanded = quote! {
        #[bitfield(#bitfield_ty)]
        #input
        #inspect_derived
    };

    TokenStream::from(expanded)
}
