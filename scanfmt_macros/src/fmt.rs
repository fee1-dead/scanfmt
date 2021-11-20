use std::collections::{hash_map::Entry, HashMap, HashSet};

use quote::quote;

use proc_macro2::{Ident, Span, TokenStream};

use syn::{parse::Parse, parse_quote, punctuated::Punctuated, Error, Expr, LitStr, Path, Token};

pub fn scan_until_literal(t: &Ident, trait_: &Path, lit: &str) -> Expr {
    let ch = lit.chars().next().unwrap();
    parse_quote! {{
        let mut chars = __string.char_indices();
        let idx = loop {
            match chars.next() {
                ::std::option::Option::Some((idx, c)) if c == #ch => {
                    break idx;
                }
                ::std::option::Option::Some(_) => {}
                ::std::option::Option::None => return Err(::scanfmt::macro_support::ScanError::Eof)
            }
        };

        let (parse, rest) = __string.split_at(idx);
        let val = <#t as #trait_>::scan(parse)?;
        (val, rest)
    }}
}

pub fn scan_until_scan(current: &Ident, trait_: &Path, next: &Ident, next_trait: &Path) -> Expr {
    parse_quote! {{
        let mut chars = __string.char_indices();
        let idx = loop {
            match chars.next() {
                ::std::option::Option::Some((idx, c)) if <#next as #next_trait>::is_valid_start(c) => {
                    break idx;
                }
                ::std::option::Option::Some(_) => {}
                ::std::option::Option::None => return Err(::scanfmt::macro_support::ScanError::Eof)
            }
        };

        let (parse, rest) = __string.split_at(idx);
        let val = <#current as #trait_>::scan(parse)?;
        (val, rest)
    }}
}

pub(crate) enum Argument {
    Implicit,
    Named(Ident),
    Index(usize),
}

pub(crate) enum Spec {
    Default,
    Octal,
    LowerHex,
    UpperHex,
    Binary,
}

#[derive(Default)]
pub(crate) struct Format {
    argument: Argument,
    spec: Spec,
}

pub(crate) enum Piece {
    Lit(String),
    Fmt(Format),
}

pub(crate) struct FormatString {
    pieces: Vec<Piece>,
    span: Span,
}

pub(crate) struct Input {
    s: Expr,
    _comma: Token![,],
    fmt: FormatString,
    _comma1: Option<Token![,]>,
    args: Punctuated<Ident, Token![,]>,
}

macro_rules! with_dollar_sign {
    ($($body:tt)*) => {
        macro_rules! __with_dollar_sign { $($body)* }
        __with_dollar_sign!($);
    }
}

// get easy to use macros by specifying the span.
macro_rules! decl_macros_with_span {
    ($sp:expr) => {
        with_dollar_sign! {
            ($d:tt) => {
                macro_rules! err {
                    ($d ($d tt:tt)*) => {
                        Error::new($sp, &format!($d ($d tt)*))
                    }
                }

                macro_rules! bail {
                    ($d ($d tt:tt)*) => {
                        return Err(err!($d ( $d tt )*))
                    }
                }
            }
        }
    };
}

impl Input {
    pub fn verify(&self) -> syn::Result<Vec<usize>> {
        let mut idents = HashMap::new();
        for (n, arg) in self.args.iter().enumerate() {
            match idents.entry(arg.clone()) {
                Entry::Occupied(prev) => {
                    let mut e = Error::new_spanned(arg, "duplicate argument");
                    e.combine(Error::new_spanned(
                        prev.key(),
                        "argument previously defined here",
                    ));
                }
                Entry::Vacant(e) => {
                    e.insert(n);
                }
            }
        }

        let mut cnt = 0;

        let mut indices = HashSet::new();
        let mut indices_vec = Vec::with_capacity(self.fmt.pieces.len());

        decl_macros_with_span!(self.fmt.span);

        for p in &self.fmt.pieces {
            if let Piece::Fmt(fmt) = p {
                let idx = fmt
                    .argument
                    .idx(&mut cnt, &idents, &self.args, self.fmt.span)?;
                if !indices.insert(idx) {
                    bail!("{} is referenced multiple times", self.args[idx]);
                }
                indices_vec.push(idx);
            }
        }

        Ok(indices_vec)
    }

    pub fn expand(self, pieceidx2argidx: &[usize]) -> TokenStream {
        let exp = self.s;
        let idents = self.args.iter();
        let spec_traits = self
            .fmt
            .pieces
            .iter()
            .filter_map(|p| {
                if let Piece::Fmt(f) = p {
                    Some(f.spec.trait_())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let (inference_type_param_idents, temp_var_idents): (Vec<_>, Vec<_>) = (0..pieceidx2argidx
            .len())
            .map(|i| {
                let var = Ident::new(&format!("__InferredVar{}", i), Span::call_site());
                let tempvar = Ident::new(&format!("__temp{}", i), Span::call_site());
                (var, tempvar)
            })
            .unzip();

        let mut fmt_counter = 0;

        let stmts = self
            .fmt
            .pieces
            .iter()
            .enumerate()
            .map(|(i, p)| match p {
                Piece::Lit(lit) => quote! {
                    __string = ::scanfmt::macro_support::advance(__string, #lit)?;
                },
                Piece::Fmt(_) => {
                    let t = &inference_type_param_idents[fmt_counter];
                    let trait_ = &spec_traits[fmt_counter];
                    let var = &temp_var_idents[fmt_counter];
                    let tokens = match self.fmt.pieces.get(i + 1) {
                        Some(Piece::Lit(lit)) => {
                            let res = scan_until_literal(t, trait_, lit);

                            quote! {
                                let (#var, __string_next) = #res;
                                __string = __string_next;
                            }
                        }
                        Some(Piece::Fmt(_)) => {
                            let next = &inference_type_param_idents[fmt_counter + 1];
                            let next_trait = &spec_traits[fmt_counter + 1];
                            let res = scan_until_scan(t, trait_, next, next_trait);

                            quote! {
                                let (#var, __string_next) = #res;
                                __string = __string_next;
                            }
                        }
                        None => quote!(let #var = <#t as #trait_>::scan(__string)?;),
                    };
                    fmt_counter += 1;
                    tokens
                }
            })
            .collect::<Vec<_>>();

        quote! {{
            fn __infer_fn< #(#inference_type_param_idents: #spec_traits),* >(
                mut __string: &str
            ) -> ::std::result::Result<(#(#inference_type_param_idents,)*), ::scanfmt::macro_support::ScanError> {
                #(#stmts)*

                Ok((#(#temp_var_idents,)*))
            }

            let ( #(#temp_var_idents,)* ) = match &#exp {
                __string => __infer_fn(*__string),
            }?;
            #(
                #idents = #temp_var_idents;
            )*
        }}
    }

    pub fn verify_and_expand(self) -> syn::Result<TokenStream> {
        let indices = self.verify()?;
        Ok(self.expand(&indices))
    }
}

impl Argument {
    pub fn idx(
        &self,
        counter: &mut usize,
        map: &HashMap<Ident, usize>,
        p: &Punctuated<Ident, Token![,]>,
        sp: Span,
    ) -> syn::Result<usize> {
        decl_macros_with_span!(sp);
        let within_bounds = |n, s| {
            if n >= p.len() {
                bail!("{} index {} is out of bounds", s, n)
            }

            Ok(n)
        };
        match self {
            Argument::Implicit => {
                let c = *counter;
                *counter += 1;
                within_bounds(c, "implicit")
            }
            Argument::Index(c) => within_bounds(*c, "specified"),
            Argument::Named(id) => map
                .get(id)
                .copied()
                .ok_or_else(|| err!("there is no argument named {}", id)),
        }
    }
}

impl Format {
    fn parse_within_braces(s: &str, sp: Span) -> syn::Result<Self> {
        decl_macros_with_span!(sp);
        if s.is_empty() {
            return Ok(Format::default());
        }
        // non-empty string;
        let col = s.find(':').unwrap_or_else(|| s.len());
        let (ident, col) = s.split_at(col);
        let col = &col[1..];

        let argument = if ident.is_empty() {
            Argument::Implicit
        } else if let Ok(index) = ident.parse() {
            Argument::Index(index)
        } else {
            Argument::Named(Ident::new(ident, sp))
        };

        let spec = if col.is_empty() {
            Spec::Default
        } else {
            match col.chars().next() {
                Some('o') => Spec::Octal,
                Some('x') => Spec::LowerHex,
                Some('X') => Spec::UpperHex,
                Some('b') => Spec::Binary,
                None => bail!("expected spec after :"),
                Some(c) => bail!(
                    "expected one of 'o', 'x', 'X', or 'b' after ':', found {}",
                    c
                ),
            }
        };

        Ok(Self { argument, spec })
    }
}

impl FormatString {
    fn parse(s: &LitStr) -> syn::Result<Self> {
        use std::mem::take;

        let span = s.span();
        decl_macros_with_span!(span);

        let s = s.value();
        let mut pieces = vec![];
        let mut next_lit = String::new();
        let mut chars = s.char_indices().peekable();
        let mut brace_start = None;
        loop {
            match chars.next() {
                Some((n, '}')) => match chars.peek() {
                    Some((_, '}')) => {
                        chars.next();
                        next_lit.push('}');
                    }
                    _ => match brace_start.take() {
                        Some(n_prev) => {
                            if !next_lit.is_empty() {
                                pieces.push(Piece::Lit(take(&mut next_lit)));
                            }
                            pieces.push(Piece::Fmt(Format::parse_within_braces(
                                &s[n_prev + 1..n],
                                span,
                            )?));
                        }
                        None => bail!("mismatched '}}' with no opening braces, index={}", n),
                    },
                },
                Some((n, '{')) => {
                    if let Some(n_prev) = brace_start {
                        bail!(
                            "attempt to start index={} while prev_index={} is not closed",
                            n,
                            n_prev,
                        )
                    }
                    match chars.peek() {
                        Some((_, '{')) => {
                            chars.next();
                            next_lit.push('{');
                        }
                        _ => brace_start = Some(n),
                    }
                }
                Some(_) if brace_start.is_some() => {}
                Some((_, c)) => {
                    next_lit.push(c);
                }
                None if brace_start.is_some() => {
                    bail!("mismatched '{{' for index={}", brace_start.unwrap())
                }
                None if !next_lit.is_empty() => {
                    pieces.push(Piece::Lit(take(&mut next_lit)));
                    break;
                }
                None => break,
            }
        }

        Ok(FormatString { pieces, span })
    }
}

impl Spec {
    pub fn trait_(&self) -> Path {
        match self {
            Spec::Default => parse_quote!(::scanfmt::macro_support::Scan),
            Spec::Binary => parse_quote!(::scanfmt::macro_support::ScanBinary),
            Spec::Octal => parse_quote!(::scanfmt::macro_support::ScanOctal),
            Spec::LowerHex => parse_quote!(::scanfmt::macro_support::ScanLowerHex),
            Spec::UpperHex => parse_quote!(::scanfmt::macro_support::ScanUpperHex),
        }
    }
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let s = input.parse()?;
        let _comma = input.parse()?;
        let fmt: LitStr = input.parse()?;
        let _comma1 = input.parse()?;
        let args = input.parse_terminated(Ident::parse)?;

        Ok(Input {
            s,
            _comma,
            fmt: FormatString::parse(&fmt)?,
            _comma1,
            args,
        })
    }
}

impl Default for Spec {
    fn default() -> Self {
        Self::Default
    }
}

impl Default for Argument {
    fn default() -> Self {
        Self::Implicit
    }
}
