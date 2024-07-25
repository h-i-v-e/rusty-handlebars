use std::fmt::Display;

use rusty_handlebars::{AsDisplay, DisplayAsHtml};

#[derive(DisplayAsHtml)]
#[Template(path="examples/templates/very-simple.hbs")]
struct Simple{}

#[derive(DisplayAsHtml)]
#[Template(path="examples/templates/more-involved.hbs")]
struct MoreInvolved<'a>{
    name: &'a str,
    age: u8
}


#[derive(DisplayAsHtml)]
#[Template(path="examples/templates/with-options.hbs")]
struct WithOptions<'a>{
    options: &'a [Option<&'a str>]
}

#[derive(DisplayAsHtml)]
#[Template(path="examples/templates/lookups.hbs")]
struct Lookups<'a>{
    names: &'a [&'a str],
    is_planet: &'a [bool]
}

struct Holder<'a>{
    held: &'a dyn Display
}

impl<'a> AsDisplay for &Holder<'a>{
    fn as_display(&self) -> impl Display {
        self.held
    }
}

#[derive(DisplayAsHtml)]
#[Template(path="examples/templates/wrapper.hbs")]
struct Wrapper<'a>{
    wrapped: &'a [Holder<'a>]
}

fn main(){
    println!("{}", Wrapper{
        wrapped: &[
            Holder{
                held: &Simple{}
            },
            Holder{
                held: &MoreInvolved{
                    name: "John",
                    age: 42
                }
            },
            Holder{
                held: &MoreInvolved{
                    name: "Boo Boo",
                    age: 0
                }
            },
            Holder{
                held: &WithOptions{
                    options: &[
                        Some("one"),
                        None,
                        Some("three")
                    ]
                }
            },
            Holder{
                held: &Lookups{
                    names: &["Mercury", "Venus", "Earth", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune", "Pluto"],
                    is_planet: &[true, true, true, true, true, true, true, true, false]
                }
            }
        ]
    }.to_string());
}
