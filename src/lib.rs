pub use hypebeast_macros::html;
use std::borrow::Cow;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = html! {
            <!DOCTYPE html>
            <html lang="en">
                <head></head>
                <body>hypebeast</body>
            </html>
        }
        .to_string();

        assert_eq!(
            result,
            r#"<!DOCTYPE html><html lang="en"><head></head><body>hypebeast</body></html>"#
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
        #[allow(non_snake_case)]
        fn Hello(name: &str) -> Component {
            html! { <div>{name}</div> }
        }

        let x = "<script>hypebeast</script>";
        let result = html! { <Hello name=x/> }.to_string();

        assert_eq!(
            result,
            r#"<div>&lt;script&gt;hypebeast&lt;/script&gt;</div>"#
        );
    }

    #[test]
    fn it_works_with_escaped_components() {
        #[allow(non_snake_case)]
        fn Hello(c: Component) -> Component {
            html! { {c} }
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
    fn it_works_with_components_with_children() {
        #[allow(non_snake_case)]
        fn Hello(name: &str, component: Component) -> Component {
            html! {
                {component}
                <div>{name}</div>
            }
        }

        let x = "hypebeast";
        let result = html! {
            <Hello name=x>
                <span>"mr."</span>
            </Hello>
        }
        .to_string();

        assert_eq!(result, r#"<span>mr.</span><div>hypebeast</div>"#);
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

        #[allow(non_snake_case)]
        fn Table(rows: Component) -> Component {
            html! { <table>{rows}</table> }
        }

        #[allow(non_snake_case)]
        fn Row(cols: Component) -> Component {
            html! { <tr>{cols}</tr> }
        }

        #[allow(non_snake_case)]
        fn Col(i: Component) -> Component {
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
    #[allow(non_snake_case)]
    fn it_works_with_multiple_children_components() {
        fn Html(component: Component) -> Component {
            html! {
                <!DOCTYPE html>
                <html lang="en">{component}</html>
            }
        }

        fn Head(component: Component) -> Component {
            html! { <head>{component}</head> }
        }

        #[allow(non_snake_case)]
        fn Body(component: Component) -> Component {
            html! { <body>{component}</body> }
        }

        let component = html! {
            <Html>
                <Head>
                    <meta name="" description=""/>
                    <title>head</title>
                </Head>
                <Body>
                    <div>hypebeast</div>
                </Body>
            </Html>
        };

        assert_eq!(component.to_string(), "<!DOCTYPE html><html lang=\"en\"><head><meta name=\"\" description=\"\"/><title>head</title></head><body><div>hypebeast</div></body></html>");
    }
}

#[derive(Debug, PartialEq, Eq)]
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

impl Render for String {
    fn render_to_string(&self, buffer: &mut String) {
        buffer.push_str(&escape(self))
    }
}

impl Render for &String {
    fn render_to_string(&self, buffer: &mut String) {
        buffer.push_str(&escape(*self))
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

impl std::fmt::Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
