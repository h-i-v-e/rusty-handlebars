use std::fmt::Display;

use rusty_handlebars::{AsDisplay, DisplayAsHtml};

struct Checklist<'a, 'b>{
    title: &'a str,
    items: Vec<ChecklistItem<'b>>
}

struct ChecklistItem<'a>{
    title: &'a str
}

struct ChecklistResponseSave{
    name: String,
    notes:  Option<String>,
    responses: Vec<bool>
}


#[derive(DisplayAsHtml)]
#[Template(path="examples/templates/email.hbs")]
struct ChecklistEmail<'a, 'b, 'c>{
    asset_title: &'a str,
    address: &'a str,
    location: &'a str,
    checklist: &'a Checklist<'b, 'c>,
    response: &'a ChecklistResponseSave,
    link: &'a str
}



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
    println!("{}", ChecklistEmail{
        asset_title: "Asset 1234",
        address: "1234 Main St",
        location: "Room 123",
        checklist: &Checklist{
            title: "Safety Checklist",
            items: vec![
                ChecklistItem{title: "Item 1"},
                ChecklistItem{title: "Item 2"},
                ChecklistItem{title: "Item 3"}
            ]
        },
        response: &ChecklistResponseSave{
            name: "John Doe".to_string(),
            notes: Some("This is a note".to_string()),
            responses: vec![true, false, true]
        },
        link: "https://example.com"
    }.to_string());
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
