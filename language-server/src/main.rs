fn main() {
    if let Err(error) = rusty_handlebars_language_server::run() {
        eprintln!("rusty-handlebars-language-server: {error}");
        std::process::exit(1);
    }
}
