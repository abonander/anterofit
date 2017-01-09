// FIXME: remove last feature flag
#![feature(proc_macro_lib, proc_macro_attribute, proc_macro_internals)]
extern crate syn;
#[macro_use] extern crate quote;
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{Tokens, ToTokens};
use syn::*;

#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_item(&input.to_string())
        .expect("Input required to contain a trait and zero or more `delegate!()` invocations");

    let mut service_trait = ServiceTrait::from_item(item);

    assert!(args.to_string().is_empty(), "#[service] attribute does not take arguments");

    service_trait.output().parse().expect("Failed to parse output")
}

struct ServiceTrait {
    name: Ident,
    vis: Visibility,
    attrs: Vec<Attribute>,
    methods: Vec<ServiceMethod>,
    delegates: Vec<Delegate>,
}

fn assert_generics_empty(generics: &Generics) {
    assert!(
        generics.lifetimes.is_empty() &&
        generics.ty_params.is_empty() &&
        generics.where_clause.predicates.is_empty(),
        "Generics are (currently) not supported on service traits"
    )
}

impl ServiceTrait {
    fn from_item(item: Item) -> Self {
        let items = if let ItemKind::Trait(unsafety, generics, bounds, items) = item.node {
            assert_eq!(unsafety, Unsafety::Normal, "Unsafe traits are not supported");
            assert_generics_empty(&generics);
            assert!(bounds.is_empty(), "Bounds are not supported on service traits");
            items
        } else {
            panic!("Target of `#[service]` attribute must be a trait");
        };

        ServiceTrait {
            name: item.ident,
            vis: item.vis,
            attrs: item.attrs,
            methods: items.into_iter().map(ServiceMethod::from_trait_item).collect(),
            delegates: vec![],
        }
    }

    fn add_delegates(&mut self, args: Vec<TokenTree>) {
        let mut args = args.into_iter().peekable();

        while args.peek().is_some() {
            self.delegates.push(Delegate::parse(&mut args));

            if let Some(token) = args.next().map(non_delimited) {
                assert_eq!(token, Token::Comma);
            }
        }
    }

    fn output(&self) -> Tokens {
        let vis = &self.vis;
        let name = &self.name;
        let attrs = &self.attrs;

        let mut out = quote! {
            #(#attrs)*
            #vis trait #name
        };

        out.append("{");

        for method in &self.methods {
            method.decl(&mut out);
        }

        out.append("}");

        if !self.delegates.is_empty() {
            unimplemented!();
        } else {
            out.append("impl<T: ::anterofit::AbsAdapter> ");
            self.name.to_tokens(&mut out);
            out.append(" for T { ");

            for method in &self.methods {
                method.method_impl("self", &mut out);
            }

            out.append(" } ");
        }

        out
    }
}

struct ServiceMethod {
    name: Ident,
    attrs: Vec<Attribute>,
    sig: MethodSig,
    body: Vec<Stmt>,
}

impl ServiceMethod {
    fn from_trait_item(trait_item: TraitItem) -> Self {
        let (sig, block) = if let TraitItemKind::Method(sig, block) = trait_item.node {
            let block = block.expect("Every trait method must have a block.");

            (sig, block)
        } else {
            panic!("Unsupported item in service trait (only methods are allowed): {:?}", trait_item)
        };

        ServiceMethod {
            name: trait_item.ident,
            attrs: trait_item.attrs,
            sig: sig,
            body: block.stmts
        }
    }

    fn header(&self, out: &mut Tokens) {
        out.append_all(&self.attrs);
        out.append("fn");
        self.name.to_tokens(out);
        self.sig.generics.to_tokens(out);
        out.append("(");
        out.append_separated(&self.sig.decl.inputs, ",");
        out.append(")");

        out.append("-> anterofit::Request");

        if let FunctionRetTy::Ty(ref ret_ty) = self.sig.decl.output {
            out.append("<");
            ret_ty.to_tokens(out);
            out.append(">");
        }
    }

    fn decl(&self, out: &mut Tokens) {
        self.header(out);
        out.append(";");
    }

    fn method_impl(&self, get_adpt: &str, out: &mut Tokens) {
        self.header(out);
        out.append("{ request_impl! { ");
        out.append(get_adpt);
        out.append(";");
        out.append_all(&self.body);
        out.append(" } } ");
    }
}

enum Delegate {
    Concrete(Ty),
    Bounded(Ty),
}

impl Delegate {
    fn parse<I: Iterator<Item = TokenTree>>(tokens: I) -> Self {
        unimplemented!()
    }
}

fn non_delimited(tt: TokenTree) -> Token {
    match tt {
        TokenTree::Token(token) => token,
        _ => panic!("Unexpected delimited token tree: {:?}", tt),
    }
}

fn unwrap_ident(token: Token) -> Ident {
    match token {
        Token::Ident(ident) => ident,
        _ => panic!("Expected identifier, got {:?}", token),
    }
}

fn delegate_type(token: Token) -> Ty {
    match token {
        Token::Ident(ident) => ident_to_type(ident),
        Token::Literal(Lit::Str(ref path, _)) => {
            let path = parse_path(path).expect("Expected path");
            unimplemented!()
        },
        _ => panic!("Expected type (bare or in string literal), got {:?}", token),
    }
}

fn ident_to_type(ident: Ident) -> Ty {
    Ty::Path(None, Path {
        global: false,
        segments: vec![PathSegment {
            ident: ident,
            parameters: PathParameters::none(),
        }]
    })
}