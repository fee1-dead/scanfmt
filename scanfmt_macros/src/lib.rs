use proc_macro::TokenStream as Ts;
use proc_macro2::TokenStream as Ts2;
use syn::parse2;

mod fmt;

#[proc_macro]
pub fn scanfmt(input: Ts) -> Ts {
    scanfmt_inner(input.into())
        .map(Into::into)
        .map_err(syn::Error::into_compile_error)
        .unwrap_or_else(Into::into)
}

fn scanfmt_inner(input: Ts2) -> syn::Result<Ts2> {
    let input: fmt::Input = parse2(input)?;
    input.verify_and_expand()
}
