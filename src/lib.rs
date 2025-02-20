#![no_std]

extern crate alloc;
extern crate self as shtml;
use alloc::{borrow::Cow, string::String, vec::Vec};
use core::fmt;
pub use shtml_macros::{component, html, Render};

#[cfg(test)]
mod tests {
    use alloc::{string::String, string::ToString, vec::Vec};
    use shtml::{component, html, Component, Render};

    //     #[test]
    //     fn it_works() {
    //         let result = html! {
    //             <!DOCTYPE html>
    //             <html lang="en">
    //                 <head></head>
    //                 <body>shtml</body>
    //             </html>
    //         }
    //         .to_string();

    //         assert_eq!(
    //             result,
    //             r#"<!DOCTYPE html><html lang="en"><head></head><body>shtml</body></html>"#
    //         );
    //     }

    //     #[test]
    //     fn it_works_with_blocks() {
    //         let x = 1;
    //         let result = html! { <div>{x}</div> }.to_string();

    //         assert_eq!(result, r#"<div>1</div>"#);
    //     }

    //     #[test]
    //     fn it_works_with_attr_blocks() {
    //         let class = "flex items-center h-full";
    //         let result = html! { <div class=class></div> }.to_string();

    //         assert_eq!(result, r#"<div class="flex items-center h-full"></div>"#);
    //     }

    //     #[test]
    //     fn it_works_with_components() {
    //         #[component]
    //         fn Hello(name: &str) {
    //             html! { <div>{name}</div> }
    //         }

    //         let x = "<script>shtml</script>";
    //         let result = html! { <Hello name=x/> }.to_string();

    //         assert_eq!(result, r#"<div>&lt;script&gt;shtml&lt;/script&gt;</div>"#);
    //     }

    //     #[test]
    //     fn it_works_with_attrs() {
    //         #[component]
    //         fn Hypermedia(target: &str) {
    //             html! { <div x-target=target></div> }
    //         }

    //         let x = "body";
    //         let result = html! { <Hypermedia target=x/> }.to_string();

    //         assert_eq!(result, r#"<div x-target="body"></div>"#);
    //     }

    //     #[test]
    //     fn it_works_with_types_that_impl_display() {
    //         #[derive(shtml::Render)]
    //         struct Example;

    //         impl core::fmt::Display for Example {
    //             fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    //                 f.write_str("example")
    //             }
    //         }

    //         let ex = Example {};
    //         let result = html! { <div x-example=ex></div> }.to_string();

    //         assert_eq!(result, r#"<div x-example="example"></div>"#);
    //     }

    //     #[test]
    //     fn it_works_with_escaped_components() {
    //         #[component]
    //         fn Hello(children: Component) {
    //             html! { {children} }
    //         }

    //         let x = "<script>alert(\"owned\")</script>";
    //         let result = html! {
    //             <Hello>
    //                 <div>{x}</div>
    //             </Hello>
    //         }
    //         .to_string();

    //         assert_eq!(
    //             result,
    //             r#"<div>&lt;script&gt;alert(&quot;owned&quot;)&lt;/script&gt;</div>"#
    //         );
    //     }

    //     #[test]
    //     fn it_works_with_components_with_attrs_and_children() {
    //         #[component]
    //         fn Heading(class: &str, children: Component) {
    //             html! { <h1 class=class>{children}</h1> }
    //         }

    //         let result = html! {
    //             <Heading class="text-7xl text-red-500">
    //                 <p>How now brown cow</p>
    //             </Heading>
    //         };

    //         assert_eq!(
    //             result.to_string(),
    //             r#"<h1 class="text-7xl text-red-500"><p>How now brown cow</p></h1>"#
    //         );
    //     }

    //     #[test]
    //     fn it_works_with_components_with_children() {
    //         #[component]
    //         fn Hello(name: &str, children: Component) {
    //             html! {
    //                 {children}
    //                 <div>{name}</div>
    //             }
    //         }

    //         let x = "shtml";
    //         let result = html! {
    //             <Hello name=x>
    //                 <span>"mr."</span>
    //             </Hello>
    //         }
    //         .to_string();

    //         assert_eq!(result, r#"<span>mr.</span><div>shtml</div>"#);
    //     }

    //     #[test]
    //     fn it_works_for_tables() {
    //         const SIZE: usize = 2;
    //         let mut rows = Vec::with_capacity(SIZE);
    //         for _ in 0..SIZE {
    //             let mut inner = Vec::with_capacity(SIZE);
    //             for i in 0..SIZE {
    //                 inner.push(i);
    //             }
    //             rows.push(inner);
    //         }

    //         let component = html! {
    //             <table>
    //                 {rows
    //                     .iter()
    //                     .map(|cols| {
    //                         html! {
    //                             <tr>
    //                                 {cols
    //                                     .iter()
    //                                     .map(|col| html! { <td>{col}</td> })}
    //                             </tr>
    //                         }
    //                     })}
    //             </table>
    //         };

    //         assert_eq!(
    //             component.to_string(),
    //             "<table><tr><td>0</td><td>1</td></tr><tr><td>0</td><td>1</td></tr></table>"
    //         );
    //     }

    //     #[test]
    //     fn it_works_for_tables_with_components() {
    //         const SIZE: usize = 2;
    //         let mut rows = Vec::with_capacity(SIZE);
    //         for _ in 0..SIZE {
    //             let mut inner = Vec::with_capacity(SIZE);
    //             for i in 0..SIZE {
    //                 inner.push(i);
    //             }
    //             rows.push(inner);
    //         }

    //         #[component]
    //         fn Table(children: Component) {
    //             html! { <table>{children}</table> }
    //         }

    //         #[component]
    //         fn Row(children: Component) {
    //             html! { <tr>{children}</tr> }
    //         }

    //         #[component]
    //         fn Col(children: Component) {
    //             html! { <td>{children}</td> }
    //         }

    //         let component = html! {
    //             <Table>
    //                 {rows
    //                     .iter()
    //                     .map(|cols| {
    //                         html! {
    //                             <Row>
    //                                 {cols.iter().map(|i| html! { <Col>{i}</Col> })}
    //                             </Row>
    //                         }
    //                     })}
    //             </Table>
    //         };

    //         assert_eq!(
    //             component.to_string(),
    //             "<table><tr><td>0</td><td>1</td></tr><tr><td>0</td><td>1</td></tr></table>"
    //         );
    //     }

    //     #[test]
    //     fn it_works_with_multiple_children_components() {
    //         #[component]
    //         fn Html(children: Component) {
    //             html! {
    //                 <!DOCTYPE html>
    //                 <html lang="en">{children}</html>
    //             }
    //         }

    //         #[component]
    //         fn Head(children: Component) {
    //             html! { <head>{children}</head> }
    //         }

    //         #[component]
    //         fn Body(children: Component) {
    //             html! { <body>{children}</body> }
    //         }

    //         let component = html! {
    //             <Html>
    //                 <Head>
    //                     <meta name="" description=""/>
    //                     <title>head</title>
    //                 </Head>
    //                 <Body>
    //                     <div>shtml</div>
    //                 </Body>
    //             </Html>
    //         };

    //         assert_eq!(component.to_string(), "<!DOCTYPE html><html lang=\"en\"><head><meta name=\"\" description=\"\"/><title>head</title></head><body><div>shtml</div></body></html>");
    //     }

    //     #[test]
    //     fn it_works_with_fragments() {
    //         #[component]
    //         fn HStack(children: Component) {
    //             html! { <div class="flex gap-4">{children}</div> }
    //         }

    //         let component = html! {
    //             <HStack>
    //                 <>
    //                     <div>1</div>
    //                     <div>2</div>
    //                     <div>3</div>
    //                 </>
    //             </HStack>
    //         };

    //         assert_eq!(
    //             component.to_string(),
    //             r#"<div class="flex gap-4"><div>1</div><div>2</div><div>3</div></div>"#
    //         );
    //     }

    //     #[test]
    //     fn it_works_with_simple_loops() {
    //         #[component]
    //         fn List(children: Component) {
    //             html! { <ul>{children}</ul> }
    //         }

    //         #[component]
    //         fn Item(children: Component) {
    //             html! { <li>{children}</li> }
    //         }

    //         let items = Vec::from([1, 2, 3]);

    //         let component = html! { <List>{items.iter().map(|i| html! { <Item>{i}</Item> })}</List> };

    //         assert_eq!(
    //             component.to_string(),
    //             r#"<ul><li>1</li><li>2</li><li>3</li></ul>"#
    //         );
    //     }

    //     #[test]
    //     fn it_works_with_fragments_and_components() {
    //         #[component]
    //         fn HStack(children: Component) {
    //             html! { <div class="flex gap-4">{children}</div> }
    //         }

    //         #[component]
    //         fn VStack(children: Component) {
    //             html! { <div class="flex flex-col gap-4">{children}</div> }
    //         }

    //         let component = html! {
    //             <HStack>
    //                 <VStack>
    //                     <div>1</div>
    //                     <div>2</div>
    //                 </VStack>
    //             </HStack>
    //         };

    //         assert_eq!(
    //             component.to_string(),
    //             r#"<div class="flex gap-4"><div class="flex flex-col gap-4"><div>1</div><div>2</div></div></div>"#
    //         );
    //     }

    //     #[test]
    //     fn it_works_with_floats() {
    //         let x = 3.14;
    //         let result = html! { <div>{x}</div> }.to_string();

    //         assert_eq!(result, r#"<div>3.14</div>"#);
    //     }

    //     #[test]
    //     fn it_works_with_special_characters() {
    //         let special_characters = "<>&\"'";
    //         let result = html! { <div>{special_characters}</div> }.to_string();

    //         assert_eq!(result, r#"<div>&lt;&gt;&amp;&quot;&#39;</div>"#);
    //     }

    //     #[test]
    //     fn it_works_with_strings() {
    //         let string = "Hi".to_string();
    //         let result = html! { <div>{string}</div> }.to_string();

    //         assert_eq!(result, r#"<div>Hi</div>"#);
    //     }

    //     #[test]
    //     fn it_works_with_string_refs() {
    //         let string_ref = &"Hi".to_string();
    //         let result = html! { <div>{string_ref}</div> }.to_string();

    //         assert_eq!(result, r#"<div>Hi</div>"#);
    //     }

    //     #[test]
    //     fn it_works_with_spread_attributes() {
    //         let attrs = Vec::from([("data-test".to_string(), "test".to_string())]);

    //         let result = html! { <div {..attrs}>Test</div> }.to_string();

    //         assert_eq!(result, r#"<div data-test="test">Test</div>"#);
    //     }

    //     #[test]
    //     fn it_works_with_out_of_order_attr_components() {
    //         #[component]
    //         fn OutOfOrderWithLifetimes(c: String, b: u8, a: &str) {
    //             html! { <div a=a b=b c=c></div> }
    //         }

    //         let result = html! { <OutOfOrderWithLifetimes b=0 c="c".into() a="a"/> }.to_string();

    //         assert_eq!(result, r#"<div a="a" b="0" c="c"></div>"#);
    //     }

    //     #[test]
    //     fn it_works_with_out_of_order_attr_components_without_refs() {
    //         #[component]
    //         fn OutOfOrder(b: u8, c: String) {
    //             html! { <div c=c b=b></div> }
    //         }
    //         let result = html! { <OutOfOrder c="c".into() b=0/> }.to_string();

    //         assert_eq!(result, r#"<div c="c" b="0"></div>"#);
    //     }

    #[test]
    fn it_works_with_optional_fields() {
        #[component]
        fn Optional(a: Option<i8>) {
            html! { <div a={a}></div> }
        }
        let result = html! { <Optional /> }.to_string();
        assert_eq!(result, r#"<div a=""></div>"#);
        let result = html! { <Optional a=Some(0) /> }.to_string();
        assert_eq!(result, r#"<div a="0"></div>"#)
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Component {
    pub html: String,
}

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

impl<T> Render for Option<T>
where
    T: Render,
{
    fn render_to_string(&self, buffer: &mut String) {
        match self {
            Some(s) => s.render_to_string(buffer),
            None => {}
        }
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

impl<I, F, T> Render for core::iter::Map<I, F>
where
    I: Iterator + Clone,
    F: FnMut(I::Item) -> T + Clone,
    T: Render,
{
    fn render_to_string(&self, buffer: &mut String) {
        for item in self.clone().into_iter() {
            item.render_to_string(buffer);
        }
    }
}

impl IntoIterator for Component {
    type Item = Component;
    type IntoIter = alloc::vec::IntoIter<Component>;

    fn into_iter(self) -> Self::IntoIter {
        alloc::vec![self].into_iter()
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

impl<T> Render for Vec<(T, T)>
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
