extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

use syn::{Body, DeriveInput, MetaItem, NestedMetaItem, Lit, Attribute, Ident, VariantData, Variant};
use quote::{Tokens};

#[proc_macro_derive(Display, attributes(display))]
pub fn display(input_stream: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let raw_input = input_stream.to_string();
    let ast = syn::parse_derive_input(&raw_input).unwrap();

    let impl_tokens = build_impl(ast);

    // Return the generated impl
    impl_tokens.parse().unwrap()
}

fn get_fmt_args(attrs: &Vec<Attribute>) -> Option<(Lit, Vec<Ident>)> {
    attrs.iter()
        .find(|attr| attr.name() == "display")
        .map(|attr| {
            use MetaItem::*;
            match attr.value {
                List(_, ref values) => {
                    if values.len() < 1 {
                        panic!(USAGE);
                    }

                    let fmt = match values[0] {
                        NestedMetaItem::MetaItem(MetaItem::NameValue(ref ident, ref s @ Lit::Str(_, _))) => {
                            if ident != "fmt" {
                                panic!(USAGE);
                            }
                            s.clone()
                        },
                        _ => {
                            panic!(USAGE);
                        },
                    };

                    // Validate possible arguments.
                    let args = values.iter().skip(1).map(|item| {

                        match item {
                            &NestedMetaItem::MetaItem(MetaItem::Word(ref field_ident)) => {
                                field_ident.clone()
                            },
                            _ => {
                                panic!(USAGE);
                            },
                        }

                    }).collect::<Vec<_>>();

                    (fmt, args)
                },
                _ => {
                    panic!(USAGE);
                },
            }
        })
}

static USAGE: &str = "display attribute has invalid format. Expected: #[display(fmt=\"..\", ...)]";

fn build_impl(ast: DeriveInput) -> Tokens {

    let ident = ast.ident;

    let code = match ast.body {
        Body::Struct(var) => {
            let self_ident = syn::Ident::from("self");
            build_struct_code(ident.clone(), self_ident, &ast.attrs, &var)
        },
        Body::Enum(variants) => {
            build_enum_impl(ident.clone(), ast.attrs, variants)
        },
    };

    quote!{
        impl ::std::fmt::Display for #ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                #code

                Ok(())
            }
        }
    }
}

fn build_enum_impl(ident: Ident, _attrs: Vec<Attribute>, variants: Vec<Variant>) -> Tokens {
    let matches: Vec<Tokens> = variants.iter().map(|var| {
        let var_ident = var.ident.clone();
        let self_ident = syn::Ident::from("item");
        let var_code = build_struct_code(ident.clone(), self_ident, &var.attrs, &var.data);


        if let Some((fmt, args)) = get_fmt_args(&var.attrs) {
            match var.data {
                VariantData::Unit => {
                    if args.len() > 0 {
                        panic!("Can not provide additional format arguments for unit variant {}::{}",
                               ident, var_ident);
                    }

                    quote!{
                        &#ident::#var_ident => { #var_code }
                    }
                },
                VariantData::Tuple(ref fields) => {

                    let field_names = fields.iter()
                        .enumerate()
                        .map(|(idx, _)| {
                            syn::Ident::from(format!("_{}", idx))
                        })
                        .collect::<Vec<_>>();

                    for arg in &args {
                        if !field_names.contains(&arg) {
                            panic!("Invalid format argument '{}' for tuple variant {}::{}. Expected underscore + field index, eg _0",
                                  arg, ident, var_ident);
                        }
                    }

                    quote!{
                        &#ident::#var_ident(#(ref #field_names),*) => {
                            write!(f, #fmt #(, #args)*)?;
                        }
                    }
                },
                VariantData::Struct(ref fields) => {

                    let field_names = fields.iter()
                        .map(|field| {
                            field.ident.as_ref().unwrap().clone()
                        })
                        .collect::<Vec<_>>();

                    for arg in &args {
                        if !field_names.contains(&arg) {
                            panic!("Invalid format argument '{}' for struct variant {}::{}. Expected name of field on struct",
                                  arg, ident, var_ident);
                        }
                    }

                    let field_names = fields.iter()
                        .map(|f| {
                            f.ident.as_ref().unwrap().clone()
                        })
                        .collect::<Vec<_>>();

                    quote!{
                        &#ident::#var_ident{#(ref #field_names),*} => {
                            write!(f, #fmt #(, #args)*)?;
                        }
                    }
                },
            }
        } else {
            let name = syn::Lit::from(var_ident.to_string());
            quote!{
                write!(f, #name)?;
            }
        }

    }).collect();

    let code = quote!{
        match self {
            #(#matches),*
        }
    };

    code
}

fn build_struct_code(ident: Ident, self_ident: Ident, attrs: &Vec<Attribute>, body: &VariantData) -> Tokens {
    let name = ident.as_ref();

    let args = get_fmt_args(&attrs);

    let mut is_tuple = false;

    let _data = match body {
        &VariantData::Struct(ref fields) => fields.clone(),
        &VariantData::Tuple(ref fields) => {
            is_tuple = true;
            fields.clone()
        },
        _ => vec![],
    };

    let code = match args {
        Some((fmt, fmt_args)) => {
            let mut args: Vec<Tokens> = fmt_args.iter().map(|arg| {

                let field = if is_tuple {
                    let mut raw = arg.to_string();
                    if raw.chars().next() == Some('_') {
                        raw = raw.chars().skip(1).collect();
                    }

                    Ident::new(raw)
                } else {
                    arg.clone()
                };

                quote!{
                    #self_ident.#field
                }
            }).collect();

            quote!{
                write!(f, #fmt #(, #args)*)?;
            }
        },
        None => {
            quote!{
                write!(f, #name)?;
            }
        },
    };

    code
}

