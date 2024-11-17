use std::{collections::HashMap, fmt::Display};

use rusty_handlebars::{AsDisplay, WithRustyHandlebars};
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

#[derive(WithRustyHandlebars)]
#[template(path="examples/templates/email.hbs")]
struct ChecklistEmail<'a, 'b, 'c>{
    asset_title: &'a str,
    address: &'a str,
    location: &'a str,
    checklist: &'a Checklist<'b, 'c>,
    response: &'a ChecklistResponseSave,
    link: &'a str
}

#[derive(WithRustyHandlebars)]
#[template(path="examples/templates/very-simple.hbs")]
struct Simple{}

#[derive(WithRustyHandlebars)]
#[template(path="examples/templates/more-involved.hbs")]
struct MoreInvolved<'a>{
    name: &'a str,
    age: u8
}


#[derive(WithRustyHandlebars)]
#[template(path="examples/templates/with-options.hbs")]
struct WithOptions<'a>{
    options: &'a [Option<&'a str>]
}

#[derive(WithRustyHandlebars)]
#[template(path="examples/templates/lookups.hbs")]
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

#[derive(WithRustyHandlebars)]
#[template(path="examples/templates/wrapper.hbs")]
struct Wrapper<'a>{
    wrapped: &'a [Holder<'a>]
}


struct Location {
    location: String,
    address: Option<String>,
}

struct Owner {
    name: String
}

struct Asset {
    code: String,
    owner: Option<Owner>,
    title: String,
    properties: HashMap<String, Option<String>>,
    location: Location
}

struct FormProperties {
    title: Option<String>,
    preamble: Option<String>
}

struct FormData {
    name: Option<String>,
    phone: Option<String>,
    message: Option<String>,
    items: Option<Vec<String>>,
    rating: Option<i32>,
}

#[derive(WithRustyHandlebars)]
#[template(path="examples/templates/map.hbs")]
struct FormTemplateData<'a> {
    properties: &'a FormProperties,
    data: &'a FormData,
    asset: &'a Asset
}

fn main(){
    println!("{}", FormTemplateData{
        properties: &FormProperties{
            preamble: Some("Please fill out the form below".to_string()),
            title: Some("Contact".to_string())
        },
        data: &FormData{
            name: Some("John Doe".to_string()),
            phone: Some("123-456-7890".to_string()),
            message: Some("Hello, world!".to_string()),
            items: Some(vec!["Item 1".to_string(), "Item 2".to_string()]),
            rating: Some(5)
        },
        asset: &Asset{
            code: "1234".to_string(),
            owner: Some(Owner{
                name: "John Doe".to_string()
            }),
            title: "Asset 1234".to_string(),
            properties: HashMap::new(),
            location: Location{
                location: "Room 123".to_string(),
                address: Some("1234 Main St".to_string())
            }
        }
    }.to_string());
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
