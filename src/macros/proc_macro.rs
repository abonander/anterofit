use proc_macro::TokenStream;
use quote::Tokens;
use syn::*;

const DEFAULT_DELEGATE_BOUND: &'static str = "::anterofit::AbsAdapter";

#[proc_macro_attribute]
pub fn service(args: Option<TokenStream>, input: TokenStream) -> TokenStream {
    let item = parse_item(input.to_string())
        .expect("Input required to contain a trait and zero or more `delegate!()` invocations");

    let mut service_trait = ServiceTrait::from_item(&item);

    if let Some(args) = args {
        let args = parse_token_trees(args.to_string()).expect("This should be infallible, right?");
        service_trait.add_delegates(args);
    }
}

struct ServiceTrait {
    name: Ident,
    vis: Visibility,
    attrs: Vec<Attribute>,
    methods: Vec<TraitMethod>,
    delegates: Vec<Delegate>,
}

impl ServiceTrait {
    fn from_item(item: Item) -> Self {
        let items = if let ItemKind::Trait(unsafety, generics, bounds, items) = item.node {
            assert_eq!(unsafety, Unsafety::Normal, "Unsafe traits are not supported");
            assert!(generics.is_empty(), "Generics are not supported on service traits");
            assert!(bounds.is_empty(), "Bounds are not supported on service traits");
            items
        } else {
            panic!("Target of `#[service]` attribute must be a trait");
        };

        ServiceTrait {
            name: item.ident,
            vis: item.vis,
            attrs: item.attrs,
            methods: items.into_iter().map(TraitMethod::from_trait_item).collect(),
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

    fn output(&self) -> TokenTree {
        quote! {
            #self.vis() trait
        }
    }
}

struct TraitMethod {
    name: Ident,
    attrs: Vec<Attribute>,
    sig: MethodSig,
    body: Vec<Stmt>,
}

impl TraitMethod {
    fn from_trait_item(trait_item: TraitItem) -> Self {
        let (sig, block) = if let TraitItemKind::Method(sig, block) = trait_item.node {
            let block = block.expect("Every trait method must have a block.");

            (sig, block)
        } else {
            panic!("Unsupported item in service trait (only methods are allowed): {}", trait_item)
        };

        TraitMethod {
            name: trait_item.ident,
            attrs: trait_item.attrs,
            sig: sig,
            body: block.stmts
        }
    }
}

enum Delegate {
    Concrete(Ty),
    Bounded(Ty),
}

impl Delegate {
    fn parse<I: Iterator<Item = TokenTree>>(tokens: I) -> Self {
        let mut tokens = tokens.map(non_delimited);

        let kind = unwrap_ident(tokens.next()
            .expect("Expected `delegate` or `delegate_bnd`", tokens));

        assert_eq!(Some(Token::Eq), tokens.next());

        let del_type = delegate_type(tokens.next().expect("Expected type name after `=`"));

        match kind {
            "delegate" => Delegate::Concrete(del_type),
            "delegate_bnd" => Delegate::Bounded(del_type),
        }
    }
}

fn non_delimited(tt: TokenTree) -> Token {
    match tt {
        TokenTree::Token(token) => token,
        _ => panic!("Unexpected delimited token tree: {}", tt),
    }
}

fn unwrap_ident(token: Token) -> Ident {
    match token {
        Token::Ident(ident) => ident,
        _ => panic!("Expected identifier, got {}", token),
    }
}

fn delegate_type(token: Token) -> Ty {
    match token {
        Token::Ident(ident) => ident_to_type(ident),
        Token::Literal(Lit::Str(ref path, _)) => {
            parse_path(path).expect()
        },
        _ => panic!("Expected type (bare or in string literal), got {}", token),
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