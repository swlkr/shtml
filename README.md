# shtml

Server-side HTML rendering for Rust using a JSX-like macro syntax. The s is silent.

`shtml` is a `no_std` crate (using `alloc`) that lets you write HTML templates directly in Rust with the `html!` macro. It supports HTML elements, components, attributes, expressions, fragments, and automatic HTML escaping.

## Installation

```
cargo add --git https://github.com/swlkr/shtml shtml
```

## Quick start

```rust
use shtml::{html, Component, Elements, Render};

let page = html! {
    <!DOCTYPE html>
    <html lang="en">
        <head><title>My Page</title></head>
        <body><h1>Hello, world!</h1></body>
    </html>
};

assert_eq!(
    page.to_string(),
    r#"<!DOCTYPE html><html lang="en"><head><title>My Page</title></head><body><h1>Hello, world!</h1></body></html>"#
);
```

## Syntax reference

### HTML elements

Standard HTML elements with literal or dynamic attributes:

```rust
use shtml::{html, Component, Render};

// Literal attributes
let result = html! { <div class="container"><p>Hello</p></div> }.to_string();
assert_eq!(result, r#"<div class="container"><p>Hello</p></div>"#);

// Dynamic attributes
let class = "flex items-center h-full";
let result = html! { <div class=class></div> }.to_string();
assert_eq!(result, r#"<div class="flex items-center h-full"></div>"#);
```

### Void elements

Self-closing elements (`<br/>`, `<img/>`, `<input/>`, etc.) are handled automatically:

```rust
# use shtml::{html, Component, Render};
let result = html! { <input type="text" disabled/> }.to_string();
assert_eq!(result, r#"<input type="text" disabled/>"#);
```

### Boolean attributes

Attributes without a value are rendered as boolean attributes:

```rust
# use shtml::{html, Component, Render};
let result = html! { <input disabled/> }.to_string();
assert_eq!(result, "<input disabled/>");
```

### Spread attributes

Use `{..expr}` to spread a `Vec<(String, String)>` as attributes on elements or components:

```rust
# use shtml::{html, Component, Render};
let attrs = Vec::from([("data-id".to_string(), "42".to_string())]);
let result = html! { <div {..attrs}>content</div> }.to_string();
assert_eq!(result, r#"<div data-id="42">content</div>"#);
```

### Expressions

Embed Rust expressions with `{expr}`. The expression must implement `Render`:

```rust
# use shtml::{html, Component, Render};
let count = 42;
let result = html! { <span>{count}</span> }.to_string();
assert_eq!(result, "<span>42</span>");

let pi = 3.14;
let result = html! { <span>{pi}</span> }.to_string();
assert_eq!(result, "<span>3.14</span>");
```

### Components

Components are PascalCase functions that return `Component`. Attributes are passed as function arguments in declaration order. Children are passed as an `Elements` parameter:

```rust
#![allow(non_snake_case)]
use shtml::{html, Component, Elements, Render};

// Component with attributes
fn Greeting(name: &str) -> Component {
    html! { <p>Hello, {name}!</p> }
}

let result = html! { <Greeting name="world"/> }.to_string();
assert_eq!(result, "<p>Hello, world!</p>");

// Component with children
fn HStack(elements: Elements) -> Component {
    html! { <div class="flex gap-4">{elements}</div> }
}

let result = html! {
    <HStack>
        <div>1</div>
        <div>2</div>
        <div>3</div>
    </HStack>
}.to_string();
assert_eq!(result, r#"<div class="flex gap-4"><div>1</div><div>2</div><div>3</div></div>"#);

// Component with attributes and children
fn Heading(class: &str, elements: Elements) -> Component {
    html! { <h1 class=class>{elements}</h1> }
}

let result = html! {
    <Heading class="text-7xl text-red-500">
        <p>How now brown cow</p>
    </Heading>
}.to_string();
assert_eq!(result, r#"<h1 class="text-7xl text-red-500"><p>How now brown cow</p></h1>"#);
```

### Module-path components

Components can be referenced by their full module path:

```rust
#![allow(non_snake_case)]
use shtml::{html, Component, Elements, Render};

mod ui {
    use super::*;
    pub fn Card(elements: Elements) -> Component {
        html! { <div class="card">{elements}</div> }
    }
}

let result = html! { <ui::Card><p>Hello</p></ui::Card> }.to_string();
assert_eq!(result, r#"<div class="card"><p>Hello</p></div>"#);
```

### Fragments

Group elements without a wrapper using `<>...</>`:

```rust
# use shtml::{html, Component, Render};
let result = html! { <><div>A</div><div>B</div></> }.to_string();
assert_eq!(result, "<div>A</div><div>B</div>");
```

### Loops / iteration

Use `.iter().map(...).collect::<Vec<_>>()` inside an expression block:

```rust
#![allow(non_snake_case)]
use shtml::{html, Component, Elements, Render};

fn List(elements: Elements) -> Component {
    html! { <ul>{elements}</ul> }
}

fn Item(elements: Elements) -> Component {
    html! { <li>{elements}</li> }
}

let items = vec![1, 2, 3];
let result = html! {
    <List>
        {items.iter().map(|i| html! { <Item>{i}</Item> }).collect::<Vec<_>>()}
    </List>
}.to_string();
assert_eq!(result, "<ul><li>1</li><li>2</li><li>3</li></ul>");
```

### HTML escaping

String content (`&str`, `String`) is automatically HTML-escaped. `Component` values are not re-escaped since they contain already-rendered HTML:

```rust
# use shtml::{html, Component, Render};
let user_input = "<script>alert(\"xss\")</script>";
let result = html! { <div>{user_input}</div> }.to_string();
assert_eq!(result, r#"<div>&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;</div>"#);
```

The `escape()` function can also be used directly:

```rust
use shtml::escape;

assert_eq!(escape("<b>bold</b>"), "&lt;b&gt;bold&lt;/b&gt;");
assert_eq!(escape("no special chars"), "no special chars"); // zero-alloc
```

Characters escaped: `<` `>` `&` `"` `'`

## `Render` trait

The core abstraction for types that can be rendered inside `html!`. Any expression in `{...}` must implement `Render`.

### Built-in implementations

| Type | Behavior |
|------|----------|
| `&str`, `String` | HTML-escaped via `escape()` |
| `Component` | Appended as-is (already rendered) |
| Integer types (`u8`, `i8`, `u16`, `i16`, `i32`, `u32`, `i64`, `u64`, `usize`, `isize`) | Formatted via `itoa` (no allocation) |
| `f32`, `f64` | Formatted via `ryu` (no allocation) |
| `Vec<T: Render>` | Each element rendered sequentially |
| `Vec<(T, T)>` | Rendered as HTML attribute pairs (` key="value"`) |

### Custom implementations

```rust
use shtml::{html, Component, Render};

struct User { name: String }

impl Render for User {
    fn render_to_string(&self, buffer: &mut String) {
        buffer.push_str(&shtml::escape(&self.name));
    }
}

let user = User { name: "Alice".into() };
let result = html! { <span>{user}</span> }.to_string();
assert_eq!(result, "<span>Alice</span>");
```

## Feature flags

### `chaos`

The `chaos` feature enables the `#[component]` attribute macro, which transforms component functions into structs. This allows attributes to be passed in any order:

```rust,ignore
use shtml::{html, component, Component, Render};

#[component]
fn Chaos(a: &str, b: u8, c: String) -> Component {
    html! { <div a=a b=b c=c></div> }
}

// Attributes in any order:
let result = html! { <Chaos b=0 c="c".into() a="a"/> }.to_string();
assert_eq!(result, r#"<div a="a" b="0" c="c"></div>"#);
```

Without `chaos`, attributes must match the function parameter order:

```rust
# #![allow(non_snake_case)]
# use shtml::{html, Component, Render};
# fn Chaos(a: &str, b: u8, c: String) -> Component {
#     html! { <div a=a b=b c=c></div> }
# }
let result = html! { <Chaos a="a" b=0 c="c".into()/> }.to_string();
```

#### Optional props

With `#[component]`, a parameter of type `Option<T>` becomes an *optional prop*. It can
be omitted at the call site (defaulting to `None`) or supplied as `Some(value)`:

```rust,ignore
use shtml::{html, component, Component, Render};

#[component]
fn Badge(text: String, count: Option<u8>) -> Component {
    match count {
        Some(c) => html! { <span>{text}{c}</span> },
        None => html! { <span>{text}</span> },
    }
}

// Provided:
let result = html! { <Badge text="hi".into() count=Some(5)/> }.to_string();
assert_eq!(result, r#"<span>hi5</span>"#);

// Skipped (defaults to None):
let result = html! { <Badge text="hi".into()/> }.to_string();
assert_eq!(result, r#"<span>hi</span>"#);
```

Optional props also work alongside children. In `chaos` mode the children parameter must
be named `elements`:

```rust,ignore
use shtml::{html, component, Component, Elements, Render};

#[component]
fn Card(title: Option<String>, elements: Elements) -> Component {
    match title {
        Some(t) => html! { <div class="card"><h2>{t}</h2>{elements}</div> },
        None => html! { <div class="card">{elements}</div> },
    }
}

let result = html! { <Card><p>body</p></Card> }.to_string();
assert_eq!(result, r#"<div class="card"><p>body</p></div>"#);
```

Non-`Option` parameters remain required; omitting one panics at build time with
`missing required prop \`name\``. Note that `Option<&str>` is not supported as a prop
type — use `Option<String>` (a by-value `Option` holding a borrow would need a named
lifetime the macro does not infer).

## Tips and tricks

- [leptosfmt](https://github.com/bram209/leptosfmt) with this override `rustfmt = { overrideCommand = ["leptosfmt", "--stdin", "--rustfmt", "--override-macro-names", "html"] }`
- [tree-sitter-rstml](https://github.com/rayliwell/tree-sitter-rstml) for html autocomplete inside of html! macros

For helix users: the html! macro should just work and have correct syntax highlighting and autocomplete with the default html lsp + tailwind if that's your jam

```toml
[language-server.tailwind-ls]
command = "tailwindcss-language-server"
args = ["--stdio"]

[language-server.tailwind-ls.config]
tailwindCSS = { experimental = { classRegex = ["class=\"(.*)\""] } }

[[language]]
name = "rust"
language-servers = ["rust-analyzer", "vscode-html-language-server", "tailwind-ls"]
```
