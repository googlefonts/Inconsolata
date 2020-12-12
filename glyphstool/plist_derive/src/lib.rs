extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields};

#[proc_macro_derive(FromPlist, attributes(rest))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let deser = add_deser(&input.data);

    let expanded = quote! {
        impl crate::from_plist::FromPlist for #name {
            fn from_plist(plist: crate::plist::Plist) -> Self {
                let mut hashmap = plist.into_hashmap();
                #name {
                    #deser
                }
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(ToPlist, attributes(rest))]
pub fn derive_to(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let ser_rest = add_ser_rest(&input.data);
    let ser = add_ser(&input.data);

    let expanded = quote! {
        impl crate::to_plist::ToPlist for #name {
            fn to_plist(self) -> crate::plist::Plist {
                #ser_rest
                #ser
                hashmap.into()
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn add_deser(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().filter_map(|f| {
                    if !is_rest(&f.attrs) {
                        let name = &f.ident;
                        let name_str = name.as_ref().unwrap().to_string();
                        let snake_name = snake_to_camel_case(&name_str);
                        Some(quote_spanned! {f.span() =>
                            #name: crate::from_plist::FromPlistOpt::from_plist(
                                hashmap.remove(#snake_name)
                            ),
                        })
                    } else {
                        None
                    }
                });
                let recurse_rest = fields.named.iter().filter_map(|f| {
                    if is_rest(&f.attrs) {
                        let name = &f.ident;
                        Some(quote_spanned! {f.span() =>
                            #name: hashmap,
                        })
                    } else {
                        None
                    }
                });
                quote! {
                    #( #recurse )*
                    #( #recurse_rest )*
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn add_ser(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().filter_map(|f| {
                    if !is_rest(&f.attrs) {
                        let name = &f.ident;
                        let name_str = name.as_ref().unwrap().to_string();
                        let snake_name = snake_to_camel_case(&name_str);
                        Some(quote_spanned! {f.span() =>
                            if let Some(plist) = crate::to_plist::ToPlistOpt::to_plist(self.#name) {
                                hashmap.insert(#snake_name.to_string(), plist);
                            }
                        })
                    } else {
                        None
                    }
                });
                quote! {
                    #( #recurse )*
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn add_ser_rest(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                for f in fields.named.iter() {
                    if is_rest(&f.attrs) {
                        let name = &f.ident;
                        return quote_spanned! { f.span() =>
                            let mut hashmap = self.#name;
                        }
                    }
                }
                quote! { let mut hashmap = HashMap::new(); }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn is_rest(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path
            .get_ident()
            .map(|ident| ident == "rest")
            .unwrap_or(false)
    })
}

fn snake_to_camel_case(id: &str) -> String {
    let mut result = String::new();
    let mut hump = false;
    for c in id.chars() {
        if c == '_' {
            hump = true;
        } else {
            if hump {
                result.push(c.to_ascii_uppercase());
            } else {
                result.push(c);
            }
            hump = false;
        }
    }
    result
}

/*
fn to_snake_case(id: &str) -> String {
    let mut result = String::new();
    for c in id.chars() {
        if c.is_ascii_uppercase() {
            result.push('_');
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}
*/
