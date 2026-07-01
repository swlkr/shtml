mod chaos;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use rstml::{self, node::Node, Parser, ParserConfig};
use std::{collections::HashSet, fmt::Debug};
use syn::{parse_macro_input, Ident, ItemFn, LitStr, Result};

/// A JSX-like macro for writing HTML templates in Rust.
///
/// See the [`shtml`](https://docs.rs/shtml) crate documentation for full syntax
/// reference and examples.
#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    match html_macro(input) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn html_macro(input: TokenStream) -> Result<TokenStream2> {
    let size_hint = input.to_string().len();
    let config = ParserConfig::new()
        .recover_block(true)
        .always_self_closed_elements(HashSet::from([
            "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
            "track", "wbr",
        ]));
    let parser = Parser::new(config);

    let nodes = parser.parse_simple(input)?;
    let buf = Ident::new("__shtml_buf", Span::call_site());
    let mut output = Output {
        buf: buf.clone(),
        static_string: String::new(),
        tokens: vec![],
    };
    for node in &nodes {
        render(&mut output, node)?;
    }

    let tokens = output.to_token_stream();

    Ok(quote! {
        {
            let mut #buf = String::with_capacity(#size_hint);
            #tokens
            Component { html: #buf }
        }
    })
}

fn render(output: &mut Output, node: &Node) -> Result<()> {
    match node {
        Node::Comment(c) => {
            output.push_str("<!--");
            output.push_str(&c.value.value());
            output.push_str("-->");
        }
        Node::Doctype(d) => {
            output.push_str("<!DOCTYPE ");
            output
                .static_string
                .push_str(&d.value.to_token_stream_string());
            output.push_str(">");
        }
        Node::Fragment(n) => {
            for node in &n.children {
                render(output, node)?;
            }
        }
        Node::Element(n) => {

            let component_name = match &n.name() {
                rstml::node::NodeName::Path(syn::ExprPath { path, .. }) => {
                    let last_segment = path.segments.last();
                    match last_segment {
                        Some(seg) => match seg.ident.to_string().get(0..1) {
                            Some(first_letter) if first_letter.to_uppercase() == first_letter => {
                                Some(path.clone())
                            }
                            _ => None,
                        },
                        None => None,
                    }
                },
                rstml::node::NodeName::Punctuated(punctuated) => {
                    let is_custom_element = punctuated.pairs().all(|pair| {
                        match pair {
                            syn::punctuated::Pair::Punctuated(_, p) => p.as_char() == '-',
                            syn::punctuated::Pair::End(_) => true,
                        }
                    });

                    if is_custom_element {
                        None
                    } else {
                        return Err(syn::Error::new_spanned(
                            punctuated,
                            format!(
                                "Unsupported element name `{}`. Punctuated element names are only supported for custom elements (e.g., `<my-element>`). Module-path components should use `::` syntax (e.g., `<module::Component>`).",
                                punctuated.to_token_stream()
                            ),
                        ));
                    }
                },
                rstml::node::NodeName::Block(_) => {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "Block expressions as element names are not supported. Use a component function or a regular HTML element name.",
                    ));
                }
            };

            match component_name {
                Some(fn_name) => {
                    let mut inputs = n
                        .open_tag
                        .attributes
                        .iter()
                        .filter_map(|attr| match attr {
                            rstml::node::NodeAttribute::Block(block) => {
                                match block {
                                    rstml::node::NodeBlock::ValidBlock(valid_block) => {
                                        for stmt in &valid_block.stmts {
                                            if let syn::Stmt::Expr(syn::Expr::Range(expr_range), _) = stmt {
                                                if let Some(box_expr) = &expr_range.end {
                                                    let tokens = (*box_expr.clone()).to_token_stream();
                                                    // Spread attributes are only meaningful for
                                                    // positional (non-chaos) component calls.
                                                    #[cfg(not(feature = "chaos"))]
                                                    { return Some(quote! { #tokens }); }
                                                    #[cfg(feature = "chaos")]
                                                    { let _ = tokens; return None; }
                                                }
                                            }
                                        }
                                        None
                                    }
                                    _ => None,
                                }
                            }
                            rstml::node::NodeAttribute::Attribute(attr) => {
                                let value = attr.value();

                                #[cfg(feature = "chaos")]
                                { let key = &attr.key; Some(quote! { .#key(#value) }) }

                                #[cfg(not(feature = "chaos"))]
                                { Some(quote! { #value }) }
                            }
                        })
                        .collect::<Vec<_>>();

                    let mut inner_output = Output::new(output.buf.clone());

                    for node in &n.children {
                        render(&mut inner_output, node)?;
                    }

                    let buf = inner_output.buf.clone();
                    let inner_tokens = inner_output.to_token_stream();

                    match inner_tokens.is_empty() {
                        false => {
                            let inner_tokens = quote! {
                                {
                                    let mut #buf = String::new();
                                    #inner_tokens
                                    Component { html: #buf }
                                }
                            };

                            // In chaos mode children are passed via the conventional
                            // `elements` setter; otherwise they are the trailing
                            // positional argument.
                            #[cfg(feature = "chaos")]
                            inputs.push(quote! { .elements(#inner_tokens) });

                            #[cfg(not(feature = "chaos"))]
                            inputs.push(inner_tokens);
                        }
                        _ => {}
                    }

                    #[cfg(feature = "chaos")]
                    let tokens = quote! { #fn_name::builder() #(#inputs)* .build() };

                    #[cfg(not(feature = "chaos"))]
                    let tokens = quote! { #fn_name(#(#inputs,)*) };

                    output.push_tokens(tokens);
                }
                None => {
                    output.push_str("<");
                    output.push_str(&n.open_tag.name.to_string());
                    for attr in &n.open_tag.attributes {
                        match attr {
                            rstml::node::NodeAttribute::Block(block) => {
                                match block {
                                    rstml::node::NodeBlock::ValidBlock(valid_block) => {
                                        for stmt in &valid_block.stmts {
                                            match stmt {
                                                syn::Stmt::Expr(expr_expr, _expr_semi) => {
                                                    match expr_expr {
                                                        syn::Expr::Range(expr_range) => {
                                                            match &expr_range.end {
                                                                Some(box_expr) => {
                                                                    let tokens = (*box_expr.clone()).to_token_stream();

                                                                    output.push_tokens(tokens);
                                                                }
                                                                _ => {}
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            },
                            rstml::node::NodeAttribute::Attribute(attr) => {
                                output.static_string.push(' ');
                                output.push_str(&attr.key.to_string());
                                match attr.value_literal_string() {
                                    Some(s) => {
                                        output.push_str("=\"");
                                        output.push_str(&s);
                                        output.push_str("\"");
                                    }
                                    None => match attr.value() {
                                        Some(expr) => {
                                            output.push_str("=\"");
                                            let tokens = expr.to_token_stream();
                                            output.push_tokens(tokens);
                                            output.push_str("\"");
                                        }
                                        None => {
                                            // TODO: bool attr?
                                        }
                                    },
                                }
                            }
                        }
                    }
                    match &n.children.is_empty() {
                        true => match &n.close_tag {
                            Some(tag) => {
                                output.push_str(">");
                                output.push_str("</");
                                output.push_str(&tag.name.to_string());
                                output.push_str(">");
                            }
                            None => {
                                output.push_str("/>");
                            }
                        },
                        false => {
                            output.push_str(">");
                            for child in &n.children {
                                render(output, child)?;
                            }

                            match &n.close_tag {
                                Some(tag) => {
                                    output.push_str("</");
                                    output.push_str(&tag.name.to_string());
                                    output.push_str(">");
                                }
                                None => {
                                    output.push_str("/>");
                                }
                            }
                        }
                    }
                }
            }
        }
        Node::Block(n) => {
            let tokens = n.to_token_stream();
            output.push_tokens(tokens);
        }
        Node::Text(n) => output.push_str(&n.value_string()),
        Node::RawText(n) => output.push_str(&n.to_token_stream_string()),
    }
    Ok(())
}

#[derive(Debug)]
struct Output {
    buf: Ident,
    static_string: String,
    tokens: Vec<TokenStream2>,
}

impl Output {
    fn new(buf: Ident) -> Self {
        Self {
            buf,
            tokens: vec![],
            static_string: String::new(),
        }
    }

    fn push_str(&mut self, string: &str) {
        self.static_string.push_str(string);
    }

    fn push_tokens(&mut self, tokens: TokenStream2) {
        self.push_expr();
        let buf = &self.buf;
        let tokens = quote! {
            #tokens.render_to_string(&mut #buf);
        };
        self.tokens.push(tokens);
    }

    fn push_expr(&mut self) {
        if self.static_string.is_empty() {
            return;
        }
        let expr = {
            let output_ident = self.buf.clone();
            let string = LitStr::new(&self.static_string, Span::call_site());
            quote!(#output_ident.push_str(#string);)
        };
        self.static_string.clear();
        self.tokens.push(expr);
    }

    fn to_token_stream(mut self) -> TokenStream2 {
        self.push_expr();
        self.tokens.into_iter().collect()
    }
}

/// Transforms a component function into a struct with named fields.
///
/// This allows component attributes to be passed in any order in the [`html!`] macro,
/// rather than requiring them to match the function parameter order.
///
/// Requires the `chaos` feature flag on the `shtml` crate.
#[proc_macro_attribute]
pub fn component(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as ItemFn);
    match chaos::component_macro(item_fn) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}
