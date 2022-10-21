//! # EventSourcing Derive
//!
//! Macro implementations for custom derivations for the *eventsourcing* crate
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::ItemStruct;
use syn::{parse, Data, DataEnum, DeriveInput, Fields, Ident, LitStr, Path, Result, Variant};

struct Field {
    field: syn::Field,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse()
    }
}

#[proc_macro_attribute]
pub fn event_meta(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(item as ItemStruct);
    let _ = parse_macro_input!(attrs as parse::Nothing);
    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        let ex: Field = syn::parse2(quote! { pub lid: LID, }).unwrap();
        fields.named.push(ex.field);
    };

    return quote! {
        #item_struct
    }
    .into();
}

/// Derives the boilerplate code for a Dispatcher
#[proc_macro_derive(Dispatcher, attributes(aggregate))]
pub fn component(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Dispatcher entry failed!");
    impl_component(&ast)
}

/// Derives the boilerplate code for an Event
#[proc_macro_derive(Event, attributes(event_type_version, event_source))]
pub fn component_event(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    //let ast: DeriveInput = syn::parse(input).expect("event parse has failed!");
    let gen = match ast.data {
        Data::Enum(ref data_enum) => impl_component_event(&ast, data_enum),
        Data::Struct(_) => quote! {
            panic!("#[derive(Event)] is only defined for enums, not structs")
        }
        .into(),
        Data::Union(_) => quote! {
            panic!("#[derive(Event)] is only defined for enums, not unions")
        }
        .into(),
    };

    gen.into()
}

fn impl_component_event(ast: &DeriveInput, data_enum: &DataEnum) -> TokenStream {
    let variants = &data_enum.variants;
    let (impl_generics, _ty_generics, where_clause) = ast.generics.split_for_impl();
    let name = &ast.ident;
    let event_type_version: Ident = ast
        .attrs
        .iter()
        .find(|attr| attr.path.segments[0].ident == "event_type_version")
        .map(|attr| attr.parse_args().unwrap())
        .unwrap_or_else(|| parse_quote!(NoSchemaVersion));

    let event_source: LitStr = ast
        .attrs
        .iter()
        .find(|attr| attr.path.segments[0].ident == "event_source")
        .map(|attr| attr.parse_args().unwrap())
        .unwrap_or_else(|| parse_quote!(NoEventSource));

    let event_matches = generate_event_matches(&name, &variants);

    let ex = quote! {
        impl #impl_generics ::eventsourcing::Event for #name #where_clause {
            fn event_type_version(&self) -> &str {
                #event_type_version
            }

            fn event_source(&self) -> &str {
                #event_source
            }

            fn event_type(&self) -> &str {
                match self {
                    #(#event_matches)*
                }
            }
        }
        impl From<::eventsourcing::cloudevents::CloudEvent> for #name {
            fn from(__source: ::eventsourcing::cloudevents::CloudEvent) -> Self {
                ::serde_json::from_str(&::serde_json::to_string(&__source.data).unwrap()).unwrap()
            }
        }
    };

    TokenStream::from(ex)
}

fn generate_event_matches(
    name: &Ident,
    variants: &Punctuated<Variant, Comma>,
) -> Vec<proc_macro2::TokenStream> {
    variants
        .iter()
        .map(|variant| {
            let id = &variant.ident;
            let et_name = event_type_name(name, id);
            match variant.fields {
                Fields::Unit => quote! {
                    #name::#id => #et_name,
                },
                Fields::Unnamed(ref fields) => {
                    let idents: Vec<_> = fields
                        .unnamed
                        .pairs()
                        .map(|p| p.value().ident.as_ref())
                        .collect();
                    quote! {
                        #name::#id( #(_ #idents,)* ) => #et_name,
                    }
                }
                Fields::Named(ref fields) => {
                    let idents: Vec<_> = fields
                        .named
                        .pairs()
                        .map(|p| p.value().ident.as_ref())
                        .collect();
                    quote! {
                        #name::#id { #(#idents: _,)* } => #et_name,
                    }
                }
            }
        })
        .collect()
}

fn event_type_name(name: &Ident, variant_id: &Ident) -> String {
    let name_s = name.to_string().to_lowercase();
    let variant_s = variant_id.to_string().to_lowercase();
    format!("{}.{}", name_s, variant_s)
}

fn impl_component(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (impl_generics, _ty_generics, where_clause) = ast.generics.split_for_impl();

    let aggregate: Path = ast
        .attrs
        .iter()
        .find(|attr| attr.path.segments[0].ident == "aggregate")
        .map(|attr| attr.parse_args().unwrap())
        .unwrap_or_else(|| parse_quote!(NoAggregate));

    quote! {
        #[async_trait]
        impl #impl_generics ::eventsourcing::Dispatcher for #name #where_clause {
            type Aggregate = #aggregate;
            type Event = <#aggregate as Aggregate>::Event;
            type Command = <#aggregate as Aggregate>::Command;
            type State = <#aggregate as Aggregate>::State;

            async fn dispatch(
               state: Self::State,
               cmd: Self::Command,
               store: impl crate::eventsourcing::prelude::EventStoreClient,
               stream: String,
               evt_meta: EventMeta,
            ) -> Vec<Result<::eventsourcing::cloudevents::CloudEvent>> {
                match Self::Aggregate::handle_command(&state, &cmd) {
                    Ok(evts) => {
                        futures::future::join_all(
                            evts.into_iter()
                                .map(|evt| store.append(evt, &stream, evt_meta.clone()))
                                .collect::<Vec<_>>(),
                        ).await
                    },
                    Err(e) => vec![Err(e)],
                }
            }
        }
    }
    .into()
}
