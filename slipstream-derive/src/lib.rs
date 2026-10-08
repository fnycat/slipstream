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
    #[darling(default)]
    pub rename: Option<String>,
    #[darling(default)]
    pub with: Option<syn::Path>,
}

impl FieldOpt {
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

    pub fn to_tokens(fields: &[FieldOpt], is_bitfield: bool) -> proc_macro2::TokenStream {
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

            f.ignore == false && !bitfield_ignore
        });

        let mut tokens = proc_macro2::TokenStream::new();
        for field in fields {
            let name = field.name();
            let accessor = field.accessor(is_bitfield);
            let config = field.config();

            tokens.append_all(quote! {
                ui.horizontal(|ui| {
                    ui.label(concat!(#name, ": "));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        const CONFIG: slipstream_shared::inspect::FieldConfig = #config;
                        let response = #accessor.draw_value(ui, &CONFIG);
                        changes |= response;
                    });
                });
                ui.end_row();
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
        let fields = FieldOpt::to_tokens(&fields, is_bitfield);
        tokens.append_all(quote! {
            /// Automatically generated by the [`Inspect`] derive macro.
            ///
            /// This function generates an abstract representation of the current struct
            // #ty is specified twice because we need to both specify the generic and the impl we want to use.
            impl slipstream_shared::inspect::Inspect for #ident {
                fn draw_inspect(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
                    use slipstream_shared::widgets;

                    let mut changes = slipstream_shared::inspect::Changes::default();
                    widgets::draw_collapsing_state(
                        widgets::CollapseDescriptor::new(
                            ui.id().with("CollapsingState"),
                            "header".into(),
                            None,
                            widgets::HeaderAlignment::Right,
                            |ui| {
                                #fields

                                Ok(())
                            }
                        ),
                        ui
                    );
                    changes
                }

                fn draw_value(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
                    let egui::CollapsingResponse { body_returned, .. } =egui::CollapsingHeader::new(#name).show(ui, |ui| {
                        let mut changes = slipstream_shared::inspect::Changes::default();
                        ui.vertical(|ui| {
                            #fields
                        });
                        changes
                    });
                    body_returned.unwrap_or_default()
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

            impl slipstream_shared::inspect::Inspect for #ident {
                fn draw_inspect(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
                    todo!();
                }

                fn draw_value(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> slipstream_shared::inspect::Changes {
                    let curr_label = slipstream_shared::inspect::AsEnumLabel::as_label(self);

                    let mut changes = slipstream_shared::inspect::Changes::default();
                    egui::ComboBox::new(ui.id().with("ComboBox"), "")
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
