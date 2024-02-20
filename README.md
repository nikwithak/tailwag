# Tailwag - The Rapid Rust Toolkit

This repo consolidates a variety of Rust libraries intended to rapidly create
scalable, Production-ready applications. The end goal of Tailwag is to enable
same-day zero-to-production development and deployments.

The current developmental version supports standing up a REST service and a
Postgres datastore for datatypes composed of primitives, timestamps, and UUIDs.
It does not currently support joins or nested data structures, however it does
provide

Sub-crates:

- **tailwag_orm**: An ORM for mapping Rust data types to database queries or
  other data stores. It currently supports creating new Postgres tables and CRUD
  operations, with experimental support for migrating the tables on changes.
- **tailwag_forms** _(experimental)_: A crate used for representing the form
  structure for creating or editing data. This exports to a format readable by
  `tailwag_react_manager` (working title), a NextJS application that pairs with
  this toolkit.
- **tailwag_web_application**: This is the primary interaction for Tailwag in
  its current state. I started building this on top of `axum`, which is itself a
  great web framework, but as I was trying to work around some limitations of
  the API, I ended up implementing my own HTTP handler.

## Project Status: Experimental

This project is being actively developed in my free time, although it is still
in early stages.

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

```rust
      fn main() {

      }
```

More examples can be found in the `/examples` folder.

## How can I support the project?

1. Use it! - Please try out the API, build some projects with it, and put it to
   the real test. As you encounter Issues as you encounter challenges or bugs.
   If you have a use case that isn't supported, please file it as an issue.

2. Stars & Likes - If you are getting value of Tailwag, consider starring or
   watching the project, or following me (@nikwithak) on GitHub. These are
   zero-cost ways to extend the audience of my work.

3. Donate or Sponsor - This project has been written entirely in my free time,
   and your donations allow me to continue devoting time and energy to
   maintaining it and developing new features. All donation amounts are accepted
   and appreciated. I would be thrilled to be able to focus on this project
   full-time, and your donations help achieve that goal. If you would like to
   sponsor this project, please email
   [nik@nikgilmore.com](mailto:nik@nikgilmore.com), I would love to chat!

4. Write some tests - I started out writing tests and adding documentation
   during development. As things picked up (and I got sidetracked with some
   compiler battles), I got a little more lax with the tests and documentation.

## Tenets [Opinionated Framework]

- [ ] Define Your Data - The data model is the most important part of an
      application, and therefore it's essential to keep the definition
      consistent and easy to modify. This framework seeks to make it simple and
      fast to define the core data structures needed for an application, and
      share that definition with other parts of the system.

- [ ] Focus on Ergonomics - Tailwag relies heavily on traits and derive macros.
      The goal is to make sensible defaults for the general case, getting a
      standard, functional app ready with zero effort while also making it easy
      to override with custom behavior.

### FAQ

##### Q: Why Rust?

My experience has taught me that the most important part of building scalable,
maintainable applications is to clearly define the data types you'll be working
with. One of the biggest traps I repeatedly see at startups is the push to move
fast instead of building a strong foundation, and it leads to unmanageable tech
debt and system fragility as the company scales.

I started learning Rust in I find Rust to be such a joy to code in - the
compiler checks my assumptions and forces me to handle errors and edge cases.
The memory safety and type safety that comes along with Rust removes the need to
worry about those things on my own. I've heard it said from more than a few
sources that "it's hard to write bad code in Rust".

I have primarily a Java and Python background, and I find Rust to be the perfect
middle-ground between the two worlds - the type safety and compiler guarantees
of Java (and then some!), but without all the verbosity.

##### Q: Why not use existing frameworks like `axum`, `actix-web`, `diesel`, `yew` or `<other_library>`?

A: I am a strong believer in practice as a productivity tool. I started these
libraries as a self-study learning exercise, and over time they've grown into
something more useful.

##### Q: What's up with the macro crates?

A: The first macros I wrote were a mess of spaghetti code, and I've slowly been
simplifying how I write them to make them more readable / maintainable. tl;dr:
They're just a mess right now.

Currently I have two macro sub-crates for the crates that have custom macros,
one for the exports and one for the logic. I thought this would make it easier
to reuse and test the macros, given the restrictions around `proc_macro`
requiring a separate crate, but so far I've just found it to be an annoying
hurdle to jump around when changing the macros and will try to simplify them in
the future.

##### Q: Why were certain decisions made?

A: Sometimes the reason a decision made is just because it's what made sense at
the time. There are some parts of the framework that I've rebuild a couple of
times, some I've been happy with the way I did it, and others that I wish I had
done differently but they are too baked in to change.

##### Q: I have another question.

A: Please [file an issue](https://github.com/nikwithak/tailwag/issues/new) or
[send me an email](mailto:nik@nikgilmore.com)!
