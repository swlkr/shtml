//! # shtml
//!
//! Server-side HTML rendering for Rust using a JSX-like macro syntax.
//!
//! `shtml` is a `no_std` crate (using `alloc`) that lets you write HTML templates
//! directly in Rust with the [`html!`] macro. It supports HTML elements, components,
//! attributes, expressions, fragments, and automatic HTML escaping.
//!
//! # Quick start
//!
//! ```
//! use shtml::{html, Component, Elements, Render};
//!
//! let page = html! {
//!     <!DOCTYPE html>
//!     <html lang="en">
//!         <head><title>My Page</title></head>
//!         <body><h1>Hello, world!</h1></body>
//!     </html>
//! };
//!
//! assert_eq!(
//!     page.to_string(),
//!     r#"<!DOCTYPE html><html lang="en"><head><title>My Page</title></head><body><h1>Hello, world!</h1></body></html>"#
//! );
//! ```
//!
//! # Components
//!
//! Components are PascalCase functions that return [`Component`]. They receive typed
//! attributes as parameters and optionally an [`Elements`] parameter for children:
//!
//! ```
//! # #![allow(non_snake_case)]
//! # use shtml::{html, Component, Elements, Render};
//! # #[cfg(not(feature = "chaos"))]
//! # fn run() {
//! fn Greeting(name: &str) -> Component {
//!     html! { <p>{name}</p> }
//! }
//!
//! let result = html! { <Greeting name="world"/> }.to_string();
//! assert_eq!(result, "<p>world</p>");
//! # }
//! # #[cfg(feature = "chaos")]
//! # fn run() {}
//! # run();
//! ```
//!
//! # HTML escaping
//!
//! String content is automatically HTML-escaped inside [`html!`]. Already-rendered
//! [`Component`] values are not re-escaped. Use [`escape()`] directly if needed.
//!
//! ```
//! # use shtml::{html, Component, Render};
//! let user_input = "<script>alert('xss')</script>";
//! let safe = html! { <div>{user_input}</div> }.to_string();
//! assert_eq!(safe, "<div>&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;</div>");
//! ```
//!
//! # Feature flags
//!
//! - **`chaos`** — Enables the [`component`] attribute macro, which transforms component
//!   functions into structs allowing attributes to be passed in any order.

#![allow(non_snake_case)]
#![no_std]

extern crate alloc;
use alloc::{borrow::Cow, string::String, vec::Vec};
use core::fmt;

/// A JSX-like macro for writing HTML templates in Rust.
///
/// The `html!` macro parses a JSX-like syntax and produces a [`Component`] containing
/// the rendered HTML string.
///
/// # Syntax
///
/// ## HTML elements
///
/// Standard HTML elements with attributes:
///
/// ```
/// # use shtml::{html, Component, Render};
/// let result = html! { <div class="container"><p>Hello</p></div> }.to_string();
/// assert_eq!(result, r#"<div class="container"><p>Hello</p></div>"#);
/// ```
///
/// ## Void elements
///
/// Self-closing elements (`<br/>`, `<img/>`, `<input/>`, etc.) are handled automatically:
///
/// ```
/// # use shtml::{html, Component, Render};
/// let result = html! { <input type="text" disabled/> }.to_string();
/// assert_eq!(result, r#"<input type="text" disabled/>"#);
/// ```
///
/// ## Dynamic attributes
///
/// Attribute values can be expressions (without curlies):
///
/// ```
/// # use shtml::{html, Component, Render};
/// let class = "flex items-center";
/// let result = html! { <div class=class></div> }.to_string();
/// assert_eq!(result, r#"<div class="flex items-center"></div>"#);
/// ```
///
/// ## Boolean attributes
///
/// Attributes without a value are rendered as boolean attributes:
///
/// ```
/// # use shtml::{html, Component, Render};
/// let result = html! { <input disabled/> }.to_string();
/// assert_eq!(result, "<input disabled/>");
/// ```
///
/// ## Spread attributes
///
/// Use `{..expr}` to spread a `Vec<(String, String)>` as attributes:
///
/// ```
/// # use shtml::{html, Component, Render};
/// # use std::vec::Vec;
/// let attrs = Vec::from([("data-id".to_string(), "42".to_string())]);
/// let result = html! { <div {..attrs}>content</div> }.to_string();
/// assert_eq!(result, r#"<div data-id="42">content</div>"#);
/// ```
///
/// ## Expressions
///
/// Embed Rust expressions with `{expr}`. The expression must implement [`Render`]:
///
/// ```
/// # use shtml::{html, Component, Render};
/// let count = 42;
/// let result = html! { <span>{count}</span> }.to_string();
/// assert_eq!(result, "<span>42</span>");
/// ```
///
/// ## Components
///
/// PascalCase names are treated as component function calls. Attributes are passed
/// as function arguments in declaration order. Children are passed as an [`Elements`]
/// parameter:
///
/// ```
/// # #![allow(non_snake_case)]
/// # use shtml::{html, Component, Elements, Render};
/// # #[cfg(not(feature = "chaos"))]
/// # fn run() {
/// fn Card(title: &str, elements: Elements) -> Component {
///     html! { <div class="card"><h2>{title}</h2>{elements}</div> }
/// }
///
/// let result = html! { <Card title="Info"><p>Details here</p></Card> }.to_string();
/// assert_eq!(result, r#"<div class="card"><h2>Info</h2><p>Details here</p></div>"#);
/// # }
/// # #[cfg(feature = "chaos")]
/// # fn run() {}
/// # run();
/// ```
///
/// ## Module-path components
///
/// Components can be referenced by their module path:
///
/// ```
/// # #![allow(non_snake_case)]
/// # use shtml::{html, Component, Elements, Render};
/// # #[cfg(not(feature = "chaos"))]
/// # fn run() {
/// mod ui {
///     use shtml::{html, Component, Elements, Render};
///     pub fn Badge(elements: Elements) -> Component {
///         html! { <span class="badge">{elements}</span> }
///     }
/// }
///
/// let result = html! { <ui::Badge>New</ui::Badge> }.to_string();
/// assert_eq!(result, r#"<span class="badge">New</span>"#);
/// # }
/// # #[cfg(feature = "chaos")]
/// # fn run() {}
/// # run();
/// ```
///
/// ## Fragments
///
/// Group elements without a wrapper using `<>...</>`:
///
/// ```
/// # use shtml::{html, Component, Render};
/// let result = html! { <><div>A</div><div>B</div></> }.to_string();
/// assert_eq!(result, "<div>A</div><div>B</div>");
/// ```
///
/// ## DOCTYPE and comments
///
/// ```
/// # use shtml::{html, Component, Render};
/// let result = html! { <!DOCTYPE html><html></html> }.to_string();
/// assert_eq!(result, "<!DOCTYPE html><html></html>");
/// ```
///
/// ## Loops / iteration
///
/// Use `.iter().map(...).collect::<Vec<_>>()` inside an expression block:
///
/// ```
/// # #![allow(non_snake_case)]
/// # use shtml::{html, Component, Elements, Render};
/// let items = vec![1, 2, 3];
/// let result = html! {
///     <ul>
///         {items.iter().map(|i| html! { <li>{i}</li> }).collect::<Vec<_>>()}
///     </ul>
/// }.to_string();
/// assert_eq!(result, "<ul><li>1</li><li>2</li><li>3</li></ul>");
/// ```
pub use shtml_macros::html;

#[cfg(not(feature = "chaos"))]
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{string::ToString, vec::Vec};

    #[test]
    fn it_works() {
        let result = html! {
            <!DOCTYPE html>
            <html lang="en">
                <head></head>
                <body>shtml</body>
            </html>
        }
        .to_string();

        assert_eq!(
            result,
            r#"<!DOCTYPE html><html lang="en"><head></head><body>shtml</body></html>"#
        );
    }

    #[test]
    fn it_works_with_blocks() {
        let x = 1;
        let result = html! { <div>{x}</div> }.to_string();

        assert_eq!(result, r#"<div>1</div>"#);
    }

    #[test]
    fn it_works_with_attr_blocks() {
        let class = "flex items-center h-full";
        let result = html! { <div class=class></div> }.to_string();

        assert_eq!(result, r#"<div class="flex items-center h-full"></div>"#);
    }

    #[test]
    fn it_works_with_components() {
        fn Hello(name: &str) -> Component {
            html! { <div>{name}</div> }
        }

        let x = "<script>shtml</script>";
        let result = html! { <Hello name=x/> }.to_string();

        assert_eq!(result, r#"<div>&lt;script&gt;shtml&lt;/script&gt;</div>"#);
    }

    #[test]
    fn it_works_with_attrs() {
        fn Hypermedia(target: &str) -> Component {
            html! { <div x-target=target></div> }
        }

        let x = "body";
        let result = html! { <Hypermedia target=x/> }.to_string();

        assert_eq!(result, r#"<div x-target="body"></div>"#);
    }

    #[test]
    fn it_works_with_escaped_components() {
        fn Hello(elements: Elements) -> Component {
            html! { {elements} }
        }

        let x = "<script>alert(\"owned\")</script>";
        let result = html! {
            <Hello>
                <div>{x}</div>
            </Hello>
        }
        .to_string();

        assert_eq!(
            result,
            r#"<div>&lt;script&gt;alert(&quot;owned&quot;)&lt;/script&gt;</div>"#
        );
    }

    #[test]
    fn it_works_with_components_with_attrs_and_children() {
        fn Heading(class: &str, els: Elements) -> Component {
            html! { <h1 class=class>{els}</h1> }
        }

        let result = html! {
            <Heading class="text-7xl text-red-500">
                <p>How now brown cow</p>
            </Heading>
        };

        assert_eq!(
            result.to_string(),
            r#"<h1 class="text-7xl text-red-500"><p>How now brown cow</p></h1>"#
        );
    }

    #[test]
    fn it_works_with_components_with_children() {
        fn Hello(name: &str, elements: Elements) -> Component {
            html! {
                {elements}
                <div>{name}</div>
            }
        }

        let x = "shtml";
        let result = html! {
            <Hello name=x>
                <span>"mr."</span>
            </Hello>
        }
        .to_string();

        assert_eq!(result, r#"<span>mr.</span><div>shtml</div>"#);
    }

    #[test]
    fn it_works_for_tables() {
        const SIZE: usize = 2;
        let mut rows = Vec::with_capacity(SIZE);
        for _ in 0..SIZE {
            let mut inner = Vec::with_capacity(SIZE);
            for i in 0..SIZE {
                inner.push(i);
            }
            rows.push(inner);
        }

        let component = html! {
            <table>
                {rows
                    .iter()
                    .map(|cols| {
                        html! {
                            <tr>
                                {cols
                                    .iter()
                                    .map(|col| html! { <td>{col}</td> })
                                    .collect::<Vec<_>>()}
                            </tr>
                        }
                    })
                    .collect::<Vec<_>>()}
            </table>
        };

        assert_eq!(
            component.to_string(),
            "<table><tr><td>0</td><td>1</td></tr><tr><td>0</td><td>1</td></tr></table>"
        );
    }

    #[test]
    fn it_works_for_tables_with_components() {
        const SIZE: usize = 2;
        let mut rows = Vec::with_capacity(SIZE);
        for _ in 0..SIZE {
            let mut inner = Vec::with_capacity(SIZE);
            for i in 0..SIZE {
                inner.push(i);
            }
            rows.push(inner);
        }

        fn Table(rows: Elements) -> Component {
            html! { <table>{rows}</table> }
        }

        fn Row(cols: Elements) -> Component {
            html! { <tr>{cols}</tr> }
        }

        fn Col(i: Elements) -> Component {
            html! { <td>{i}</td> }
        }

        let component = html! {
            <Table>
                {rows
                    .iter()
                    .map(|cols| {
                        html! {
                            <Row>
                                {cols.iter().map(|i| html! { <Col>{i}</Col> }).collect::<Vec<_>>()}
                            </Row>
                        }
                    })
                    .collect::<Vec<_>>()}
            </Table>
        };

        assert_eq!(
            component.to_string(),
            "<table><tr><td>0</td><td>1</td></tr><tr><td>0</td><td>1</td></tr></table>"
        );
    }

    #[test]
    fn it_works_with_multiple_children_components() {
        fn Html(component: Elements) -> Component {
            html! {
                <!DOCTYPE html>
                <html lang="en">{component}</html>
            }
        }

        fn Head(component: Elements) -> Component {
            html! { <head>{component}</head> }
        }

        fn Body(component: Elements) -> Component {
            html! { <body>{component}</body> }
        }

        let component = html! {
            <Html>
                <Head>
                    <meta name="" description=""/>
                    <title>head</title>
                </Head>
                <Body>
                    <div>shtml</div>
                </Body>
            </Html>
        };

        assert_eq!(component.to_string(), "<!DOCTYPE html><html lang=\"en\"><head><meta name=\"\" description=\"\"/><title>head</title></head><body><div>shtml</div></body></html>");
    }

    #[test]
    fn it_works_with_fragments() {
        fn HStack(elements: Elements) -> Component {
            html! { <div class="flex gap-4">{elements}</div> }
        }

        let component = html! {
            <HStack>
                <>
                    <div>1</div>
                    <div>2</div>
                    <div>3</div>
                </>
            </HStack>
        };

        assert_eq!(
            component.to_string(),
            r#"<div class="flex gap-4"><div>1</div><div>2</div><div>3</div></div>"#
        );
    }

    #[test]
    fn it_works_with_simple_loops() {
        fn List(elements: Elements) -> Component {
            html! { <ul>{elements}</ul> }
        }

        fn Item(elements: Elements) -> Component {
            html! { <li>{elements}</li> }
        }

        let items = Vec::from([1, 2, 3]);

        let component = html! { <List>{items.iter().map(|i| html! { <Item>{i}</Item> }).collect::<Vec<_>>()}</List> };

        assert_eq!(
            component.to_string(),
            r#"<ul><li>1</li><li>2</li><li>3</li></ul>"#
        );
    }

    #[test]
    fn it_works_with_fragments_and_components() {
        fn HStack(elements: Elements) -> Component {
            html! { <div class="flex gap-4">{elements}</div> }
        }

        fn VStack(elements: Elements) -> Component {
            html! { <div class="flex flex-col gap-4">{elements}</div> }
        }

        let component = html! {
            <HStack>
                <VStack>
                    <div>1</div>
                    <div>2</div>
                </VStack>
            </HStack>
        };

        assert_eq!(
            component.to_string(),
            r#"<div class="flex gap-4"><div class="flex flex-col gap-4"><div>1</div><div>2</div></div></div>"#
        );
    }

    #[test]
    fn it_works_with_floats() {
        let x = 3.14;
        let result = html! { <div>{x}</div> }.to_string();

        assert_eq!(result, r#"<div>3.14</div>"#);
    }

    #[test]
    fn it_works_with_special_characters() {
        let special_characters = "<>&\"'";
        let result = html! { <div>{special_characters}</div> }.to_string();

        assert_eq!(result, r#"<div>&lt;&gt;&amp;&quot;&#39;</div>"#);
    }

    #[test]
    fn it_works_with_strings() {
        let string = "Hi".to_string();
        let result = html! { <div>{string}</div> }.to_string();

        assert_eq!(result, r#"<div>Hi</div>"#);
    }

    #[test]
    fn it_works_with_string_refs() {
        let string_ref = &"Hi".to_string();
        let result = html! { <div>{string_ref}</div> }.to_string();

        assert_eq!(result, r#"<div>Hi</div>"#);
    }

    #[test]
    fn it_works_with_spread_attributes() {
        let attrs = Vec::from([ ("data-test".to_string(), "test".to_string() ) ]);

        let result = html! { <div {..attrs}>Test</div> }.to_string();

        assert_eq!(result, r#"<div data-test="test">Test</div>"#);
    }

    #[test]
    fn it_works_with_bool_attributes() {
        let result = html! { <input disabled/> }.to_string();
        assert_eq!(result, r#"<input disabled/>"#);
    }

    #[test]
    fn it_works_with_bool_and_value_attributes() {
        let result = html! { <input type="text" disabled/> }.to_string();
        assert_eq!(result, r#"<input type="text" disabled/>"#);
    }

    #[test]
    fn it_works_with_module_path_components() {
        mod components {
            use super::*;
            pub fn Card(elements: Elements) -> Component {
                html! { <div class="card">{elements}</div> }
            }
        }

        let result = html! { <components::Card><p>Hello</p></components::Card> }.to_string();
        assert_eq!(result, r#"<div class="card"><p>Hello</p></div>"#);
    }

    #[test]
    fn it_works_with_spread_attributes_on_components() {
        fn Link(attrs: Vec<(String, String)>, elements: Elements) -> Component {
            let href = attrs.iter().find(|(k, _)| k == "href").map(|(_, v)| v.as_str()).unwrap_or("");
            html! { <a href=href>{elements}</a> }
        }

        let attrs = Vec::from([("href".to_string(), "/home".to_string())]);
        let result = html! { <Link {..attrs}>Home</Link> }.to_string();
        assert_eq!(result, r#"<a href="/home">Home</a>"#);
    }
}

/// A type alias for [`Component`], used as the parameter type for component children.
///
/// When a component accepts children (content placed between its opening and closing tags),
/// it declares an `elements: Elements` parameter. The macro automatically collects and
/// renders the children into a `Component` and passes it as this argument.
///
/// # Example
///
/// ```
/// # #![allow(non_snake_case)]
/// # use shtml::{html, Component, Elements, Render};
/// # #[cfg(not(feature = "chaos"))]
/// # fn run() {
/// fn Wrapper(elements: Elements) -> Component {
///     html! { <div class="wrapper">{elements}</div> }
/// }
///
/// let result = html! {
///     <Wrapper>
///         <p>Child content</p>
///     </Wrapper>
/// }.to_string();
/// assert_eq!(result, r#"<div class="wrapper"><p>Child content</p></div>"#);
/// # }
/// # #[cfg(feature = "chaos")]
/// # fn run() {}
/// # run();
/// ```
pub type Elements = Component;

/// A rendered HTML string.
///
/// `Component` is the primary output type of the [`html!`] macro. It wraps a `String`
/// containing pre-rendered HTML. When a `Component` is embedded inside another [`html!`]
/// call, its content is inserted as-is without re-escaping.
///
/// # Creating a `Component`
///
/// Components are created via the [`html!`] macro:
///
/// ```
/// # use shtml::{html, Component, Render};
/// let component = html! { <h1>Hello</h1> };
/// assert_eq!(component.to_string(), "<h1>Hello</h1>");
/// ```
///
/// # Fields
///
/// - `html` — The rendered HTML string, accessible directly.
///
/// # Trait implementations
///
/// - [`Display`](core::fmt::Display) — Outputs the HTML string.
/// - [`Render`] — Appends the HTML to a buffer without escaping (already rendered).
/// - [`Clone`], [`Debug`], [`PartialEq`], [`Eq`]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Component {
    pub html: String,
}

/// The core trait for types that can be rendered inside [`html!`].
///
/// Any expression used inside `{...}` in the `html!` macro must implement `Render`.
/// The [`render_to_string`](Render::render_to_string) method appends the rendered
/// representation to the given buffer.
///
/// # Built-in implementations
///
/// | Type | Behavior |
/// |------|----------|
/// | `&str`, `String` | HTML-escaped via [`escape()`] |
/// | `Component` | Appended as-is (already rendered) |
/// | Integer types (`u8`, `i32`, `usize`, etc.) | Formatted via [`itoa`] |
/// | `f32`, `f64` | Formatted via [`ryu`] |
/// | `Vec<T: Render>` | Each element rendered sequentially |
/// | `Vec<(T, T)>` | Rendered as HTML attribute pairs (`key="value"`) |
///
/// # Implementing `Render` for a custom type
///
/// ```
/// # use shtml::{html, Component, Render};
/// struct User { name: String }
///
/// impl Render for User {
///     fn render_to_string(&self, buffer: &mut String) {
///         // Escape user-provided content for safety
///         buffer.push_str(&shtml::escape(&self.name));
///     }
/// }
///
/// let user = User { name: "Alice".into() };
/// let result = html! { <span>{user}</span> }.to_string();
/// assert_eq!(result, "<span>Alice</span>");
/// ```
pub trait Render {
    fn render_to_string(&self, buffer: &mut String);
}

macro_rules! impl_render_int {
    ($t:ty) => {
        impl Render for $t {
            fn render_to_string(&self, buffer: &mut String) {
                let mut b = itoa::Buffer::new();
                buffer.push_str(b.format(*self));
            }
        }
    };
}

macro_rules! impl_render_float {
    ($t:ty) => {
        impl Render for $t {
            fn render_to_string(&self, buffer: &mut String) {
                let mut b = ryu::Buffer::new();
                buffer.push_str(b.format(*self));
            }
        }
    };
}

impl_render_int!(u8);
impl_render_int!(i8);
impl_render_int!(u16);
impl_render_int!(i16);
impl_render_int!(i64);
impl_render_int!(u64);
impl_render_int!(i32);
impl_render_int!(u32);
impl_render_int!(usize);
impl_render_int!(isize);

impl_render_float!(f64);
impl_render_float!(f32);

impl Render for Component {
    fn render_to_string(&self, buffer: &mut String) {
        buffer.push_str(&self.html);
    }
}

impl Render for String {
    fn render_to_string(&self, buffer: &mut String) {
        buffer.push_str(&escape(self))
    }
}

impl Render for &str {
    fn render_to_string(&self, buffer: &mut String) {
        buffer.push_str(&escape(*self))
    }
}

impl<T> Render for Vec<T>
where
    T: Render,
{
    fn render_to_string(&self, buffer: &mut String) {
        self.iter().for_each(|s| s.render_to_string(buffer));
    }
}

impl <T> Render for Vec<(T, T)>
where
    T: Render,
{
    fn render_to_string(&self, buffer: &mut String) {
        self.iter().for_each(|(key, value)| {
            buffer.push_str(" ");
            key.render_to_string(buffer);
            buffer.push_str("=");
            buffer.push_str(r#"""#);
            value.render_to_string(buffer);
            buffer.push_str(r#"""#);
        });
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.html))
    }
}

/// Escapes HTML special characters in a string.
///
/// Replaces the following characters with their HTML entity equivalents:
///
/// | Character | Entity |
/// |-----------|--------|
/// | `<` | `&lt;` |
/// | `>` | `&gt;` |
/// | `&` | `&amp;` |
/// | `"` | `&quot;` |
/// | `'` | `&#39;` |
///
/// Returns a [`Cow<str>`] — if no escaping is needed, the original string is returned
/// without allocation.
///
/// # Example
///
/// ```
/// use shtml::escape;
///
/// assert_eq!(escape("<b>bold</b>"), "&lt;b&gt;bold&lt;/b&gt;");
/// assert_eq!(escape("no special chars"), "no special chars"); // no allocation
/// ```
pub fn escape<'a, S: Into<Cow<'a, str>>>(input: S) -> Cow<'a, str> {
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

/// An attribute macro that transforms a component function into a struct, allowing
/// attributes to be passed in any order and optional props to be skipped.
///
/// Without `#[component]`, attributes must be passed in the same order as the function
/// parameters. With `#[component]`, the macro generates a builder, so attributes can be
/// specified in any order.
///
/// Requires the `chaos` feature flag.
///
/// # Optional props
///
/// A parameter of type `Option<T>` is an *optional prop*: it may be omitted at the call
/// site (defaulting to `None`), or supplied as `Some(value)`.
///
/// ```ignore
/// use shtml::{html, component, Component, Render};
///
/// #[component]
/// fn Badge(text: String, count: Option<u8>) -> Component {
///     match count {
///         Some(c) => html! { <span>{text}{c}</span> },
///         None => html! { <span>{text}</span> },
///     }
/// }
///
/// // Both are valid:
/// let with = html! { <Badge text="hi".into() count=Some(5)/> }.to_string();
/// let without = html! { <Badge text="hi".into()/> }.to_string();
/// ```
///
/// # Example
///
/// ```ignore
/// use shtml::{html, component, Component, Render};
///
/// #[component]
/// fn Button(label: &str, disabled: u8) -> Component {
///     html! { <button disabled=disabled>{label}</button> }
/// }
///
/// // Attributes in any order:
/// let result = html! { <Button disabled=0 label="Click"/> }.to_string();
/// ```
#[cfg(feature = "chaos")]
pub use shtml_macros::component;

#[cfg(feature = "chaos")]
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn it_works_with_out_of_order_attr_components() {
        #[component]
        fn Chaos(c: String, b: u8, a: &str) -> Component {
            html! { <div a=a b=b c=c></div> }
        }

        let result = html! { <Chaos b=0 c="c".into() a="a"/> }.to_string();

        assert_eq!(result, r#"<div a="a" b="0" c="c"></div>"#);
    }

    #[test]
    fn it_works_with_out_of_order_attr_components_without_refs() {
        #[component]
        fn Chaos(b: u8, c: String) -> Component {
            html! { <div c=c b=b></div> }
        }
        let result = html! { <Chaos c="c".into() b=0/> }.to_string();

        assert_eq!(result, r#"<div c="c" b="0"></div>"#);
    }

    #[component]
    fn Badge(text: String, count: Option<u8>) -> Component {
        match count {
            Some(c) => html! { <span>{text}{c}</span> },
            None => html! { <span>{text}</span> },
        }
    }

    #[test]
    fn it_works_with_optional_prop_present() {
        let result = html! { <Badge text="hi".into() count=Some(5)/> }.to_string();
        assert_eq!(result, r#"<span>hi5</span>"#);
    }

    #[test]
    fn it_works_with_optional_prop_absent() {
        let result = html! { <Badge text="hi".into()/> }.to_string();
        assert_eq!(result, r#"<span>hi</span>"#);
    }

    #[test]
    fn it_works_with_optional_prop_and_other_order() {
        let result = html! { <Badge count=Some(7) text="yo".into()/> }.to_string();
        assert_eq!(result, r#"<span>yo7</span>"#);
    }

    #[test]
    fn it_works_with_optional_prop_and_children() {
        #[component]
        fn Card(title: Option<String>, elements: Elements) -> Component {
            match title {
                Some(t) => html! { <div class="card"><h2>{t}</h2>{elements}</div> },
                None => html! { <div class="card">{elements}</div> },
            }
        }

        let with_title = html! { <Card title=Some("Info".into())><p>body</p></Card> }.to_string();
        assert_eq!(
            with_title,
            r#"<div class="card"><h2>Info</h2><p>body</p></div>"#
        );

        let without_title = html! { <Card><p>body</p></Card> }.to_string();
        assert_eq!(without_title, r#"<div class="card"><p>body</p></div>"#);
    }
}
