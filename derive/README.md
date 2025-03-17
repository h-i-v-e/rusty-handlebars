# Rusty Handlebars Derive

A procedural macro crate for the `rusty-handlebars` templating engine that provides derive macros for easy template integration.

## Features

- Derive macro for implementing Handlebars templating
- Compile-time template validation
- Type-safe template processing
- Automatic template path resolution
- Support for nested structs and collections
- HTML safety through derive macros

## Usage

### Basic Usage

```rust
use rusty_handlebars::WithRustyHandlebars;

#[derive(WithRustyHandlebars)]
#[template(path = "templates/hello.hbs")]
struct HelloTemplate<'a> {
    name: &'a str,
}
```

### Complex Types

```rust
use rusty_handlebars::WithRustyHandlebars;

#[derive(WithRustyHandlebars)]
#[template(path = "templates/user_profile.hbs")]
struct UserProfileTemplate<'a> {
    user: User<'a>,
    posts: Vec<Post<'a>>,
    is_admin: bool,
    last_login: Option<&'a str>,
}

struct User<'a> {
    name: &'a str,
    email: &'a str,
    role: UserRole,
    preferences: Option<UserPreferences<'a>>,
}

struct Post<'a> {
    title: &'a str,
    content: &'a str,
    created_at: &'a str,
    tags: Vec<&'a str>,
    author: User<'a>,
}
```

### Collections and Options

```rust
use rusty_handlebars::WithRustyHandlebars;

#[derive(WithRustyHandlebars)]
#[template(path = "templates/dashboard.hbs")]
struct DashboardTemplate<'a> {
    stats: Vec<Stat<'a>>,
    recent_activity: Vec<Activity<'a>>,
    notifications: Option<Vec<Notification<'a>>>,
}

struct Stat<'a> {
    label: &'a str,
    value: f64,
    trend: Option<Trend<'a>>,
}
```

### Web Framework Integration

```rust
use actix_web::{web, HttpResponse, Responder};
use rusty_handlebars::WithRustyHandlebars;

#[derive(WithRustyHandlebars)]
#[template(path = "templates/page.hbs")]
struct PageTemplate<'a> {
    title: &'a str,
    content: &'a str,
    user: Option<User<'a>>,
}

async fn handle_page() -> impl Responder {
    let template = PageTemplate {
        title: "Welcome",
        content: "Hello, World!",
        user: Some(User {
            name: "John Doe",
            email: "john@example.com",
            role: UserRole::User,
            preferences: None,
        }),
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(template.to_string())
}
```

## Template Path Resolution

Template paths are always relative to the workspace root.

```rust
// Path relative to workspace root
#[template(path = "templates/hello.hbs")]
```

## Supported Types

The derive macro supports:
- Basic types (&str, numbers, bool)
- Optional types (Option)
- Custom types implementing AsDisplay and AsDisplayHtml

## HTML Safety

The derive macro automatically handles HTML safety:

```rust
#[derive(WithRustyHandlebars)]
#[template(path = "templates/safe.hbs")]
struct SafeTemplate<'a> {
    // Regular text (HTML escaped)
    text: &'a str,
    
    // HTML content (not escaped)
    html: &'a str,
}
```

```handlebars
<!-- templates/safe.hbs -->
<div>
    <p>{{text}}</p>  <!-- HTML escaped -->
    <div>{{{html}}}</div>  <!-- Not escaped -->
</div>
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details. 