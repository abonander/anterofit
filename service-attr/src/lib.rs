#![feature(proc_macro_attribute)]
extern crate syn;
#[macro_use] extern crate quote;
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{Tokens, ToTokens};
use syn::*;

use std::iter::Peekable;

#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_item(&input.to_string())
        .expect("Input required to contain a trait and zero or more `delegate!()` invocations");

    let service_trait = ServiceTrait::from_item(item);

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

        let (methods, delegates) = collect_items(items);

        ServiceTrait {
            name: item.ident,
            vis: item.vis,
            attrs: item.attrs,
            methods: methods,
            delegates: delegates,
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
            for delegate in &self.delegates {
                delegate.output(&self.name, &self.methods, &mut out);
            }
        } else {
            let self_ = parse_token_trees("self").unwrap();

            out.append("impl<T: ::anterofit::AbsAdapter> ");
            self.name.to_tokens(&mut out);
            out.append(" for T { ");

            for method in &self.methods {
                method.method_impl(&self_, &mut out);
            }

            out.append(" } ");
        }

        out
    }
}

fn collect_items(items: Vec<TraitItem>) -> (Vec<ServiceMethod>, Vec<Delegate>) {
    let mut methods = vec![];
    let mut delegates = vec![];

    for item in items {
        match item.node {
            TraitItemKind::Method(..) => methods.push(ServiceMethod::from_trait_item(item)),
            TraitItemKind::Macro(mac) => delegates.push(Delegate::from_mac(mac)),
            _ => panic!("Unsupported item in service trait: {:?}", item),
        }
    }

    (methods, delegates)
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

    fn method_impl(&self, get_adpt: &[TokenTree], out: &mut Tokens) {
        self.header(out);
        out.append("{ request_impl! { ");
        out.append_all(get_adpt);
        out.append(";");
        out.append_all(&self.body);
        out.append(" } } ");
    }
}

struct Delegate {
    generics: Vec<TokenTree>,
    for_type: Vec<TokenTree>,
    where_clause: Vec<TokenTree>,
    get_adpt: Vec<TokenTree>,
}

impl Delegate {
    fn from_mac(mac: Mac) -> Self {
        assert_eq!(mac.path, "delegate".into(), "Only `delegate!()` macro invocations are allowed \
                                                 inside service traits.");
        Self::parse(mac.tts)
    }

    fn parse(mut tokens: Vec<TokenTree>) -> Self {
        let tokens = match tokens.pop() {
            Some(TokenTree::Delimited(delimited)) => delimited.tts,
            None => panic!("Empty `delegate!()` invocation!"),
            Some(token) =>  {
                tokens.push(token);
                panic!("Unsupported `delegate!()` invocation: {:?}", tokens)
            },
        };

        let mut parser = DelegateParser(tokens.into_iter().peekable());

        parser.expect_keyword("impl");

        let generics = parser.get_generics();

        parser.expect_keyword("for");

        let for_type = parser.get_type();

        assert!(!for_type.is_empty(), "Expected type, got nothing");

        let where_clause = parser.get_where();

        let get_adpt = parser.get_body_inner();

        Delegate {
            generics: generics,
            for_type: for_type,
            where_clause: where_clause,
            get_adpt: get_adpt,
        }
    }

    fn output(&self, trait_name: &Ident, methods: &[ServiceMethod], out: &mut Tokens) {
        out.append("impl");
        out.append_all(&self.generics);
        trait_name.to_tokens(out);
        out.append("for");
        out.append_all(&self.for_type);
        out.append_all(&self.where_clause);
        out.append("{");

        for method in methods {
            method.method_impl(&self.get_adpt, out);
        }

        out.append("}");
    }
}

struct DelegateParser<I: Iterator>(Peekable<I>);

impl<I: Iterator<Item = TokenTree>> DelegateParser<I> {

    fn expect_keyword(&mut self, expect: &str) {
        match self.0.next() {
            Some(TokenTree::Token(Token::Ident(ref ident))) => assert_eq!(ident.as_ref(), expect),
            Some(other) => panic!("Expected keyword {:?}, got {:?}", expect, other),
            None => panic!("Expected keyword/ {:?}, found nothing"),
        }
    }

    fn get_generics(&mut self) -> Vec<TokenTree> {
        let mut depth = 0;

        let ret = self.take_while(|token| {
            match *token {
                TokenTree::Token(Token::Lt) => depth += 1,
                TokenTree::Token(Token::Gt) => {
                    depth -= 1;

                    if depth == 0 { return true; }
                },
                _ => (),
            }

            depth > 0
        });

        if depth != 0 {
            panic!("Missing closing > on generics in delegate!() invocation: {:?}", ret);
        }

        ret
    }

    fn get_type(&mut self) -> Vec<TokenTree> {
        self.take_while(|token| match *token {
            TokenTree::Delimited(ref delimited) => delimited.delim != DelimToken::Brace,
            TokenTree::Token(Token::Ident(ref ident)) => ident != "where",
            _ => true,
        })
    }

    fn get_where(&mut self) -> Vec<TokenTree> {
        match self.0.peek() {
            Some(&TokenTree::Token(Token::Ident(ref ident))) if ident == "where" => (),
            _ => return vec![],
        }

        self.take_while(|token| match *token {
            TokenTree::Delimited(ref delimited) => delimited.delim != DelimToken::Brace,
            _ => true,
        })
    }

    fn get_body_inner(&mut self) -> Vec<TokenTree> {
        let delimited = match self.0.next() {
            Some(TokenTree::Delimited(delimited)) => delimited,
            Some(TokenTree::Token(token)) => panic!("Expected opening brace, got {:?}", token),
            None => panic!("Expected opening brace, got nothing"),
        };

        assert_eq!(delimited.delim, DelimToken::Brace);

        delimited.tts
    }

    fn take_while<F>(&mut self, mut predicate: F) -> Vec<TokenTree> where F: FnMut(&TokenTree) -> bool {
        let mut out = vec![];

        loop {
            if !self.0.peek().map_or(false, &mut predicate) {
                break
            }

            out.push(self.0.next().unwrap())
        }

        out
    }
}