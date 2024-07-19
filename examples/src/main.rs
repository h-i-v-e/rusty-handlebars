use rusty_handlebars::ToHtml;

#[derive(ToHtml)]
#[Template(path="examples/templates/very-simple.hbs")]
struct Simple{}

#[derive(ToHtml)]
#[Template(path="examples/templates/more-involved.hbs")]
struct MoreInvolved<'a>{
    name: &'a str,
    age: u8
}

fn main(){
    println!("{}", Simple{});
    println!("{}", MoreInvolved{
        name: "John",
        age: 42
    });
    println!("{}", MoreInvolved{
        name: "Boo Boo",
        age: 0
    });
}