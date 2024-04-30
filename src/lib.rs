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
                <body>"hypebeast"</body>
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
        let result = html! { <Hello name=x/> }.render_to_string();

        assert_eq!(
            result,
            r#"<div>&lt;script&gt;hypebeast&lt;/script&gt;</div>"#
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
        .render_to_string();

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
            component.render_to_string(),
            "<table><tr><td>0</td><td>1</td></tr><tr><td>0</td><td>1</td></tr></table>"
        );
    }
}

#[macro_export]
macro_rules! raw {
    ($expr:expr) => {
        $expr
    };
}

#[derive(Debug, PartialEq, Eq)]
pub struct Component {
    pub html: String,
}

pub trait Render {
    fn render_to_string(&self) -> String;
}

macro_rules! impl_render_to_string {
    ($t:ty) => {
        impl Render for $t {
            fn render_to_string(&self) -> String {
                self.to_string()
            }
        }
    };
}

impl_render_to_string!(u8);
impl_render_to_string!(i8);
impl_render_to_string!(u16);
impl_render_to_string!(i16);
impl_render_to_string!(f64);
impl_render_to_string!(f32);
impl_render_to_string!(i64);
impl_render_to_string!(u64);
impl_render_to_string!(i32);
impl_render_to_string!(u32);
impl_render_to_string!(usize);
impl_render_to_string!(isize);

impl Render for Component {
    fn render_to_string(&self) -> String {
        self.html.clone()
    }
}

impl Render for String {
    fn render_to_string(&self) -> String {
        escape(self).into()
    }
}

impl Render for &str {
    fn render_to_string(&self) -> String {
        escape(*self).into()
    }
}

impl<T> Render for Vec<T>
where
    T: Render,
{
    fn render_to_string(&self) -> String {
        self.iter()
            .map(|s| s.render_to_string())
            .collect::<Vec<_>>()
            .join("")
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
