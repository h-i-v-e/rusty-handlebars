fn main() {
    if std::env::args().nth(1).as_deref() == Some("--version") {
        println!(
            "rusty-handlebars-language-server {}",
            env!("CARGO_PKG_VERSION")
        );
        return;
    }
    if let Err(error) = rusty_handlebars_language_server::run() {
        eprintln!("rusty-handlebars-language-server: {error}");
        std::process::exit(1);
    }
}
