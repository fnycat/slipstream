use darling::ast::Data;
use darling::{FromDeriveInput, FromField};
use heck::ToTitleCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{ToTokens, TokenStreamExt, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Ident, Token, Type, Visibility};
use syn::{PathSegment, Result};

#[derive(Debug, darling::FromField)]
#[darling(attributes(inspect))]
struct FieldOpt {
    pub ident: Option<Ident>,
    pub ty: Type,

    #[darling(default)]
    pub degrees: bool,
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
    pub fn accessor(&self) -> Ident {
        match &self.ident {
            Some(x) => x.clone(),
            None => todo!(),
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
            degrees,
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

        quote! {
            slipstream_shared::inspect::FieldConfig {
                label: #label,
                category: #category,
                read_only: #read_only,
                range: #range,
                degrees: #degrees
            }
        }
    }
}

#[derive(Debug, darling::FromVariant)]
struct FieldVariant {
    pub ident: Ident,

    #[darling(default)]
    pub rename: Option<String>,
    #[darling(default)]
    pub ignore: bool,
}

impl FieldVariant {}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(inspect))]
struct Input {
    pub ident: Ident,
    pub data: darling::ast::Data<FieldVariant, FieldOpt>,

    #[darling(default)]
    pub label: Option<String>,
}

impl Input {
    fn fields_to_tokens(&self, fields: &[FieldOpt]) -> proc_macro2::TokenStream {
        let fields = fields.iter().filter(|f| f.ignore == false);
        let mut tokens = quote! {
            let mut inner_response = None;
        };

        for field in fields {
            let name = field.name();
            let accessor = field.accessor();
            let config = field.config();

            tokens.append_all(quote! {
                ui.horizontal(|ui| {
                    ui.label(concat!(#name, ": "));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        if let Some(res) = &mut inner_response {
                            *res |= self.#accessor.draw_value(ui, &#config);
                        } else {
                            inner_response = Some(self.#accessor.draw_value(ui, &#config));
                        }
                    });
                });
                ui.end_row();
            });
        }

        tokens.append_all(quote! {
            inner_response.expect("property window response was empty")
        });

        tokens
    }

    fn struct_to_tokens(&self, fields: &[FieldOpt], tokens: &mut proc_macro2::TokenStream) {
        let Self { ident, label, .. } = self;

        let fields = self.fields_to_tokens(&fields);
        tokens.append_all(quote! {
            /// Automatically generated by the [`Inspect`] derive macro.
            ///
            /// This function generates an abstract representation of the current struct
            // #ty is specified twice because we need to both specify the generic and the impl we want to use.
            impl slipstream_shared::inspect::Inspect for #ident {
                fn draw_inspect(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> egui::Response {
                    egui::Grid::new(ui.id().with(concat!(stringify!(#ident), "_properties")))
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            #fields
                        }).response
                }

                fn draw_value(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> egui::Response {
                    let egui::InnerResponse { inner, .. } = ui.vertical(|ui| {
                        #fields
                    });
                    inner
                }
            }
        });
    }

    fn enum_to_tokens(&self, variants: &[FieldVariant], tokens: &mut proc_macro2::TokenStream) {
        let Self { ident, label, .. } = self;

        tokens.append_all(quote! {
            impl slipstream_shared::inspect::Inspect for #ident {
                fn draw_inspect(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> egui::Response {

                }

                fn draw_value(&mut self, ui: &mut egui::Ui, cfg: &slipstream_shared::inspect::FieldConfig) -> egui::Response {
                    egui::
                }
            }
        })
    }
}

impl ToTokens for Input {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match &self.data {
            Data::Struct(data) => self.struct_to_tokens(&data.fields, tokens),
            Data::Enum(data) => self.enum_to_tokens(&data, tokens),
        }
    }
}

/// Fields not marked with `pub` will not be shown in the property window by default.
/// This can be overriden with `hidden = false`, the attribute will always take priority over field
/// visibility.
#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn derive_inspect(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let input = Input::from_derive_input(&input).expect("failed to parse input");
    let tokens = input.into_token_stream();

    TokenStream::from(tokens)
}
