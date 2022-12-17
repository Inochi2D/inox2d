use darling::{ast, FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(node), forward_attrs(node_state), supports(struct_any))]
struct NodeInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), NodeFieldReceiver>,
    name: Option<syn::LitStr>,
}

#[allow(unused)]
#[derive(Debug, FromField)]
#[darling(forward_attrs(node_state))]
struct NodeFieldReceiver {
    ident: Option<syn::Ident>,
    attrs: Vec<syn::Attribute>,
}

#[proc_macro_derive(Node, attributes(node_state))]
pub fn derive_node(input: TokenStream) -> TokenStream {
    impl_node(&syn::parse(input).unwrap())
}

fn impl_node(input: &syn::DeriveInput) -> TokenStream {
    let receiver = NodeInputReceiver::from_derive_input(&input).unwrap();

    let NodeInputReceiver {
        ref ident,
        ref generics,
        ref data,
        ref name,
    } = receiver;

    let (imp, ty, wher) = generics.split_for_impl();
    let fields = data.as_ref().take_struct().unwrap().fields;

    // Get fields that have the #[node_state] attribute
    let annotated_fields = fields
        .into_iter()
        .enumerate()
        .filter_map(|(i, field)| {
            // support for unnamed fields
            let ident = field.ident.as_ref().map(|v| quote!(#v)).unwrap_or_else(|| {
                let i = syn::Index::from(i);
                quote!(#i)
            });

            Some(ident).filter(|_| field.attrs.len() == 1)
        })
        .collect::<Vec<_>>();

    let Some(annotated_field) = annotated_fields.get(0) else {
        panic!("Must have exactly 1 field annotated with #[node_state]");
    };

    let typetag_proc = match name {
        Some(name) => quote! { #[typetag::serde(name = #name)] },
        None => quote! { #[typetag::serde] },
    };

    let gen = quote! {
        #typetag_proc
        impl #imp ::inox2d::nodes::node::Node for #ident #ty #wher {
            fn get_node_state(&self) -> &::inox2d::nodes::node::NodeState {
                &self.#annotated_field
            }

            fn get_node_state_mut(&mut self) -> &mut ::inox2d::nodes::node::NodeState {
                &mut self.#annotated_field
            }

            fn as_any(&self) -> &dyn ::core::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn ::core::any::Any {
                self
            }
        }
    };

    gen.into()
}

#[allow(unused)]
fn main() {
    parse_input(
        quote! {
            #[derive(Node)]
            #[node(name = "bruh")]
            pub struct Foo {
                bar: bool,

                #[node_state]
                baz: i64,
            }
        }
        .into(),
    );

    parse_input(
        quote! {
            #[derive(Node)]
            #[node]
            pub struct Foo {
                #[nope]
                bar: bool,

                #[node_state]
                baz: i64,
            }
        }
        .into(),
    );

    parse_input(
        quote! {
            #[derive(Node)]
            #[node(name = "wtf")]
            pub struct Foo(bool, #[node_state] i64);
        }
        .into(),
    );
}

fn parse_input(input: TokenStream) {
    eprintln!("==============================\n");
    eprintln!("INPUT:\n\n{input}\n");
    let tokens = derive_node(input);
    eprintln!("EMITS:\n\n{tokens}\n");
}
