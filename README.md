# Tailwag - The Rapid Rust Toolkit

This repo consolidates a variety of Rust libraries intended to rapidly create
REST APIs and data applications. The end goal of Tailwag is to enable same-day
zero-to-production development and deployments for new services, with sensible
defaults.

The current developmental version supports standing up a REST service and a
Postgres datastore for datatypes composed of primitives, timestamps, and UUIDs.
It does currently support joins or nested data structures, except for limited
One-to-one data structure support.

Sub-crates:

- **tailwag_orm**: An ORM for mapping Rust data types to database queries or
  other data stores. It currently supports creating new Postgres tables and CRUD
  operations, with experimental support for migrating the tables on changes.
- **tailwag_forms** \_: A crate used for representing the form structure for
  creating or editing data. This exports to a format readable by
  [`tailwag_react_manager`](https://github.com/nikwithak/tailwag_react_manager),
  a NextJS application that pairs with this toolkit to generate management form.

- **tailwag_web_application**: This is the primary interaction for Tailwag in
  its current state. I started building this on top of `axum`, which is itself a
  great web framework, but as I was trying to work around some of its
  limitations, I ended up implementing my own HTTP handler, and finally removed
  `axum` as a dependency.

## Project Status: Experimental

This project is being actively developed in my free time, although it is still
in early stages. It currently has a large number of `unwrap()` calls, `TODO`
comments, and unoptimized code. I've made an effort to document known technical
debt as comments in the code. The macros in particular are a bit funky (see
FAQ - "What's up with the macros?").

I've made made some recent improvements to the error handling and hope soon to
eliminate all `.unwrap()` calls, which currently cause unexpected crashes in
some cases.

It is also currently single-threaded, only able to handle one request at a time,
which is due to the WIP status of the underlying HTTP implementation.

## Getting Started

Tailwag's most useful feature right now is in standing up a REST API for a set
of defined data types. The following code example will stand up an API with full
CRUD operations for the `Item` struct:

> **NOTE** In order to run the example, you will need to have a Postgres
> instance running locally. If no environment config is found, Tailwag will use
> `postgres:postgres` as the username:password in development mode.
>
> ```
> docker run --name some-postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_USER=postgres -p 5432:5432 postgres
> ```

#### Minimal Working Example

The following is an entire Rust program build on Tailwag. This chunk of code
creates a REST webservice running on `http://localhost:8081`, with the `/todo/`
endpoint supporting CRUD operations through `POST`, `GET`, `PATCH`, and `DELETE`
HTTP methods. `/todo/{id}` is also supported for GET.

Note that tailwag has some dependencies that are not current re-exported by
tailwag. Let's create a project from scratch to get our first Tailwag
application:

First, create a new Cargo project:

```bash
cargo new my_tailwag_app
cd my_tailwag_app
```

Next add our dependencies:

```bash
cargo add tailwag
cargo add serde --features derive # Required for just about all tailwag operations.
cargo add chrono # If you need datetime support.
cargo add uuid # Required for tailwag database support.
cargo add sqlx # Required for database communication with Postgres.
cargo add tokio --features full # Required for the async runtime.
```

Now our application:

```rust
use tailwag::macros::derive_magic;
use tailwag::web::application::WebService;

#[tokio::main]
async fn main() {
    derive_magic! {
        struct Todo {
            id: uuid::Uuid, // Note: Currently all data types MUST have an `id` of type `uuid::Uuid`. A future version will remove this limitation.
            title: String,
            description: String,
            due_date: chrono::NaiveDateTime,
        }
    }
    WebService::builder("Todo Service")
        .with_resource::<Todo>()
        .build_service()
        .run()
        .await
        .expect("Web service crashed.");
}
```

    The `derive_magic!` macro derives a lot of traits that the `WebService` struct
    uses for building routes, postgres data, etc.

Don't forget to start your postgres instance:

```bash
docker run --name some-postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_USER=postgres -p 5432:5432 postgres
```

Now run the service:

```bash
cargo run
```

Hit the endpoint with some TODOs:

```bash
curl -X POST http://localhost:8081/todo \
    -H "Content-Type: application/json" \
    -d '{"title": "Add multithread support", "description": "To be viable in a Production environment, tailwag needs to support multiple concurrent requests.", "due_date": "2024-06-13T00:00:00"}'

curl -X POST http://localhost:8081/todo \
    -H "Content-Type: application/json" \
    -d '{"title": "Remove id requirement", "description": "Update tailwag to allow more flexibility for table primary keys.", "due_date": "2024-06-13T00:00:00"}'
```

Now let's GET all the TODOs:

```bash
curl http://localhost:8081/todo | jq
[
  {
    "id": "68e20f9e-0fbd-4bda-9095-b2749948579d",
    "title": "Add multithread support",
    "description": "To be viable in a Production environment, tailwag needs to support multiple concurrent requests.",
    "due_date": "2024-06-13T00:00:00"
  },
  {
    "id": "505180d4-eb25-4352-8599-6e4d69bd2806",
    "title": "Remove id requirement",
    "description": "Update tailwag to allow more flexibility for table primary keys.",
    "due_date": "2024-06-13T00:00:00"
  }
]
```

Or just a single TODO:

```bash
curl http://localhost:8081/todo | jq
{
  "id": "68e20f9e-0fbd-4bda-9095-b2749948579d",
  "title": "Add multithread support",
  "description": "To be viable in a Production environment, tailwag needs to support multiple concurrent requests.",
  "due_date": "2024-06-13T00:00:00"
}
```

This and more detailed examples can be found in the `/examples` folder. There
are also some more detailed examples in the
[`tailwag_web_service`](https://github.com/nikwithak/tailwag_web_service) crate.

## How can I support the project?

1. Use it! - Please try out the API, build some projects with it, and put it to
   the real test. Please file issues for any challenges or bugs you encounter.
   If you have a use case that isn't supported, please file it as an issue.

2. Stars & Likes - Consider starring or watching the project, and/or following
   me ([@nikwithak](https://github.com/nikwithak)) on GitHub. These are
   zero-cost ways to extend the audience of my work.

3. Donate or Sponsor - This project has been written entirely in my free time,
   and your donations allow me to continue devoting time and energy to
   maintaining it and developing new features. All donation amounts are accepted
   and appreciated. I would be thrilled to be able to focus on this project
   full-time, and your donations help achieve that goal.

   If you would like to sponsor this project, please email
   [nik@nikgilmore.com](mailto:nik@nikgilmore.com), I would love to chat!

## Tenets [Opinionated Framework]

Tailwag is an opinionated framework, and will usher you into specific patterns.
I plan to shortly publish [The Tailwag Book](#), a free and open source user
guide to building applications with Tailwag.

Tailwag focuses on the following tenets as the building blocks for your basic
application:

- [ ] Define Your Data

  - The data model is the most important part of an application, and therefore
    it's essential to keep the definition consistent and easy to modify. This
    framework seeks to make it simple and fast to define the core data
    structures needed for an application, and share that definition with other
    parts of the system.
  - Actions are defined on an object, or a group of objects. These are exposed
    as webhooks / API endpoints to trigger actions.
  - The logic and

- [ ] Focus on Ergonomics - Tailwag relies heavily on traits and derive macros.
      The goal is to make sensible defaults for the general case, getting a
      standard, functional app ready with zero effort while also making it easy
      to override with custom behavior through trait implementations. In
      general, I have tried to avoid procedural (non-derive) macros that perform
      too much magic.

### FAQ

##### Q: Why not use existing crates like `axum`, `actix-web`, `diesel`, `yew` or `<other_library>`?

A: I am a strong believer in practice as a productivity tool. I started these
libraries as a self-study learning exercise, and over time they've grown into
something more useful.

##### Q: What's up with the macro crates?

A: The first macros I wrote were a mess of spaghetti code, and I've slowly been
simplifying how I write them to make them more readable / maintainable. tl;dr:
They're just a mess right now.

Currently I have two macro sub-crates for the crates that have custom macros,
one for the exports and one for the logic. I initially thought this would make
it easier to reuse and test the macros, given the compiler-enforced requirement
that `proc_macro` exports be an entirely separate crate - but the reality is
that I just created an annoying hurdle to jump around when changing the macros.
I will be refactoring and simplify those macros in the future.

##### Q: Why were certain decisions made?

A: Sometimes the reason a decision made is just because it's what made sense at
the time. There are some parts of the framework that I've rebuild a couple of
times, some I've been happy with the way I did it at first, and others that I
wish I had done differently but they are too baked in at this point to change
easily. Such is the struggle of all software projects.

##### Q: I have another question.

A: Please [file an issue](https://github.com/nikwithak/tailwag/issues/new) or
[send me an email](mailto:nik@nikgilmore.com)!

##### Can I use Tailwag in Production?

A: You are free to use Tailwag however you see fit, although it has some known
limitations at this time. It is provided "as is", without warranty of any kind.
If you use it in a Production application, make sure you test extensively.

## LICENSE

Copyright (c) 2024 Nik Gilmore (@nikwithak)

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
