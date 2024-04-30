use std::{borrow::Cow, collections::HashSet, fmt::Debug};

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use rstml::{self, node::Node, parse2, Parser, ParserConfig};
use syn::{
    parse::{Parse, ParseStream, Peek},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, ExprCall, ExprInfer, PatPath, Result, Token,
};

#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    match html_macro(input) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn html_macro(input: TokenStream) -> Result<TokenStream2> {
    let config = ParserConfig::new()
        .recover_block(true)
        .always_self_closed_elements(HashSet::from([
            "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
            "track", "wbr",
        ]));
    let parser = Parser::new(config);

    let nodes = parser.parse_simple(input)?;
    // dbg!(&nodes);
    let mut output = Output {
        format_string: String::new(),
        tokens: vec![],
    };
    nodes
        .into_iter()
        .for_each(|node| render(&mut output, &node));
    let Output {
        format_string,
        tokens,
    } = output;

    Ok(match tokens.is_empty() {
        true => quote! { Component { html: #format_string.into() } },
        false => quote! { Component { html: format!(#format_string, #(#tokens,)*) } },
    })
}

fn render(output: &mut Output, node: &Node) {
    match node {
        Node::Comment(c) => {
            output.format_string.push_str("<!--");
            output.format_string.push_str(&escape(c.value.value()));
            output.format_string.push_str("-->");
        }
        Node::Doctype(d) => {
            output.format_string.push_str("<!DOCTYPE ");
            output
                .format_string
                .push_str(&d.value.to_token_stream_string());
            output.format_string.push_str(">");
        }
        Node::Fragment(_) => todo!(),
        Node::Element(n) => {
            let component_name = match &n.name() {
                rstml::node::NodeName::Path(syn::ExprPath { path, .. }) => match path.get_ident() {
                    Some(ident) => match ident.to_string().get(0..1) {
                        Some(first_letter) => match first_letter.to_uppercase() == first_letter {
                            true => Some(ident),
                            false => None,
                        },
                        None => None,
                    },
                    None => todo!(),
                },
                rstml::node::NodeName::Punctuated(_) => todo!(),
                rstml::node::NodeName::Block(_) => todo!(),
            };
            match component_name {
                Some(fn_name) => {
                    output.format_string.push_str("{}");

                    let mut inputs = n
                        .open_tag
                        .attributes
                        .iter()
                        .map(|attr| match attr {
                            rstml::node::NodeAttribute::Block(_) => todo!(),
                            rstml::node::NodeAttribute::Attribute(attr) => {
                                let value = attr.value();
                                quote! { #value }
                            }
                        })
                        .collect::<Vec<_>>();

                    let mut inner_output = Output {
                        format_string: String::new(),
                        tokens: vec![],
                    };

                    for node in &n.children {
                        render(&mut inner_output, &node);
                    }

                    let Output {
                        format_string,
                        tokens,
                    } = inner_output;

                    match (format_string.is_empty(), tokens.is_empty()) {
                        (false, true) => {
                            inputs.push(quote! { Component { html: #format_string.into() } });
                        }
                        (false, false) => {
                            inputs.push(quote! { Component { html: format!(#format_string, #(#tokens,)*) } });
                        }
                        _ => {}
                    }

                    let tokens = quote! { #fn_name(#(#inputs,)*) };

                    output.tokens.push(tokens);
                }
                None => {
                    output.format_string.push_str("<");
                    output.format_string.push_str(&n.open_tag.name.to_string());
                    for attr in &n.open_tag.attributes {
                        match attr {
                            rstml::node::NodeAttribute::Block(_) => todo!(),
                            rstml::node::NodeAttribute::Attribute(attr) => {
                                output.format_string.push(' ');
                                output.format_string.push_str(&attr.key.to_string());
                                match attr.value_literal_string() {
                                    Some(s) => {
                                        output.format_string.push_str("=\"");
                                        output.format_string.push_str(&escape(&s));
                                        output.format_string.push_str("\"");
                                    }
                                    None => match attr.value() {
                                        Some(expr) => {
                                            output.format_string.push_str("=\"{}\"");
                                            output.tokens.push(expr.to_token_stream());
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
                                output.format_string.push_str(">");
                                output.format_string.push_str("</");
                                output.format_string.push_str(&tag.name.to_string());
                                output.format_string.push_str(">");
                            }
                            None => {
                                output.format_string.push_str("/>");
                            }
                        },
                        false => {
                            output.format_string.push_str(">");
                            for child in &n.children {
                                render(output, &child);
                            }

                            match &n.close_tag {
                                Some(tag) => {
                                    output.format_string.push_str("</");
                                    output.format_string.push_str(&tag.name.to_string());
                                    output.format_string.push_str(">");
                                }
                                None => {
                                    output.format_string.push_str("/>");
                                }
                            }
                        }
                    }
                }
            }
        }
        Node::Block(n) => {
            output.format_string.push_str("{}");
            let tokens = n.to_token_stream();
            output.tokens.push(quote! { #tokens.render_to_string() });
        }
        Node::Text(n) => output.format_string.push_str(&escape(&n.value_string())),
        Node::RawText(n) => todo!(),
    }
}

fn escape<'a, S: Into<Cow<'a, str>>>(input: S) -> Cow<'a, str> {
    let input = input.into();
    fn needs_escaping(c: char) -> bool {
        c == '<' || c == '>' || c == '&' || c == '"' || c == '\''
    }

    if let Some(first) = input.find(needs_escaping) {
        let mut output = String::from(&input[0..first]);
        output.reserve(input.len() - first);
        let rest = input[first..].chars();
        for c in rest {
            match c {
                '<' => output.push_str("&lt;"),
                '>' => output.push_str("&gt;"),
                '&' => output.push_str("&amp;"),
                '"' => output.push_str("&quot;"),
                '\'' => output.push_str("&#39;"),
                _ => output.push(c),
            }
        }
        Cow::Owned(output)
    } else {
        input
    }
}

#[derive(Debug)]
struct Output {
    format_string: String,
    tokens: Vec<TokenStream2>,
}
