# shtml

shtml is a rust library for rendering html.

## Installation

```
cargo add https://github.com/swlkr/shtml
```

## Examples

Just write or copy/paste plain old html

```rust
use shtml::{Elements, Component};

let result = html! {
    <!DOCTYPE html>
    <html lang="en">
        <head></head>
        <body>shtml the s is silent</body>
    </html>
}
.to_string();
```

Get this back in the result var

```html
<!DOCTYPE html>
<html lang="en">
    <head></head>
    <body>shtml the s is silent</body>
</html>
```

Attrs work like you would expect

```rust
let class = "flex items-center h-full";
let result = html! { <div class=class></div> }.to_string();

// <div class="flex items-center h-full"></div>
```

Pass in rust exprs in curlies just make sure they impl `Render`

```rust
let x = 1;
let result = html! { <div>{x}</div> }.to_string();

// <div>1</div>
```

Strings get escaped

```rust
let x = "<script>alert(\"pwned\")</script>";
let result = html! { <div>{x}</div> }.to_string();

// <div>&lt;script&gt;alert(&quotpwned&quot)&lt;/script&gt;</div>
```

Components work like jsx

```rust
#![allow(non_snake_case)]

fn HStack(elements: Elements) -> Component {
    html! { <div class="flex gap-4">{elements}</div> }
}

let component = html! {
    <HStack>
      <div>1</div>
      <div>2</div>
      <div>3</div>
    </HStack>
}.to_string();

// <div class="flex gap-4"><div>1</div><div>2</div><div>3</div></div>
```

Attrs with components work as well

```rust
#![allow(non_snake_case)]

fn Hypermedia(target: &str) -> Component {
    html! { <div x-target=target></div> }
}

let x = "body";
let result = html! { <Hypermedia target=x/> }.to_string();

// <div x-target="body"></div>
```

Nested components

```rust
#![allow(non_snake_case)]

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
}.to_string();

// <div class="flex gap-4"><div class="flex flex-col gap-4"><div>1</div><div>2</div></div></div>
```

Attrs + nested components

```rust
fn Heading(class: &str, els: Elements) -> Component {
    html! { <h1 class=class>{els}</h1> }
}

let result = html! {
    <Heading class="text-7xl text-red-500">
        <p>How now brown cow</p>
    </Heading>
}.to_string();

// <h1 class="text-7xl text-red-500"><p>How now brown cow</p></h1>
```

Fragments just pass through their children

```rust
#![allow(non_snake_case)]

fn HStack(elements: Elements) -> Component {
    html! { <div class="flex gap-4">{elements}</div> }
}

fn VStack(elements: Elements) -> Component {
    html! { <div class="flex flex-col gap-4">{elements}</div> }
}

let component = html! {
    <HStack>
      <>
        <VStack>
            <div>1</div>
            <div>2</div>
        </VStack>
      </>
    </HStack>
}.to_string();

// <div class="flex gap-4"><div class="flex flex-col gap-4"><div>1</div><div>2</div></div></div>
```

The `Render` trait is only implemented for `Vec<T: Render>`

```rust
#![allow(non_snake_case)]

fn List(elements: Elements) -> Component {
    html! { <ul>{elements}</ul> }
}

fn Item(elements: Elements) -> Component {
    html! { <li>{elements}</li> }
}

let items = vec![1, 2, 3];

let result = html! {
  <List>
    {
      items
        .iter()
        .map(|i| html! {
          <Item>{i}</Item>
        })
        .collect::<Vec<_>>()
    }
  </List>
}.to_string();

// <ul><li>1</li><li>2</li><li>3</li></ul>
```

# Tips and tricks

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
