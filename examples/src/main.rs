use rusty_handlebars::DisplayAsHtml;

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
    println!("{}", WithOptions{
        options: &[
            Some("one"),
            None,
            Some("three")
        ]
    });
}