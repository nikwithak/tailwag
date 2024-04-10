# Tailwag - The Rapid Rust Toolkit

This repo consolidates a variety of Rust libraries intended to rapidly create
scalable, Production-ready applications. The end goal of Tailwag is to enable
same-day zero-to-production development and deployments for new services, with
sensible defaults.

The current developmental version supports standing up a REST service and a
Postgres datastore for datatypes composed of primitives, timestamps, and UUIDs.
It does not currently support joins or nested data structures.

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
  the API, I ended up implementing my own HTTP handler, and finally removed
  `axum` as a dependency.

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

#### Minimal Working Example

The following is an entire Rust program build on Tailwag. This chunk of code
creates a REST webservice running on `http://localhost:8081`, with the
`/events/` endpoint supporting CRUD operations through `POST`, `GET`, `PATCH`,
and `DELETE` HTTP methods. `/events/{id}` is also supported for GET.

```rust
      fn main() {

      }
```

More examples can be found in the `/examples` folder.

## How can I support the project?

1. Use it! - Please try out the API, build some projects with it, and put it to
   the real test. Please file issues for any challenges or bugs you encounter.
   If you have a use case that isn't supported, please file it as an issue.

2. Stars & Likes - If you are getting value of Tailwag, consider starring or
   watching the project, and/or following me (@nikwithak) on GitHub. These are
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
      general, I have trie to avoid procedural (non-derive) macros that perform
      too much magic.

### FAQ

##### Q: Why Rust?

My experience has taught me that the most important part of building scalable,
maintainable applications is to clearly define the data types you'll be working
with. One of the biggest traps I have repeatedly seen startups fall into is a
push to move fast instead of investing in a strong foundation. Hacked together
prototypes become the foundation for multi-million dollar businesses, and those
same business ignore security standards until their Series B investors start
asking for SOC compliance. It is my personal opinion that this is a dangerous
and unsustainable business model.

Allowing willynillyness for core system can lead to unmanageable tech debt and
system fragility as the company scales. Rust's strong typing and memory safety
make this easy to manage and force developers to check our assumptions when
coding, instead of when investigating Production incidents.

##### Q: Why not use existing frameworks like `axum`, `actix-web`, `diesel`, `yew` or `<other_library>`?

A: I am a strong believer in practice as a productivity tool. I started these
libraries as a self-study learning exercise, and over time they've grown into
something more useful.

I have used many of these libraries and highly recommend them - this is merely a
separate project.

##### Q: What's up with the macro crates?

A: The first macros I wrote were a mess of spaghetti code, and I've slowly been
simplifying how I write them to make them more readable / maintainable. tl;dr:
They're just a mess right now.

Currently I have two macro sub-crates for the crates that have custom macros,
one for the exports and one for the logic. I initially thought this would make
it easier to reuse and test the macros, given the compiler-enforced requirement
that `proc_macro` exports be an entirely separate crate - but so far I've just
created another annoying hurdle to jump around when changing the macros. I will
be refactoring and simplify those macros in the future.

##### Q: Why were certain decisions made?

A: Sometimes the reason a decision made is just because it's what made sense at
the time. There are some parts of the framework that I've rebuild a couple of
times, some I've been happy with the way I did it at first, and others that I
wish I had done differently but they are too baked in at this point to change
easily. Such is the struggle of all software projects.

##### Q: I have another question.

A: Please [file an issue](https://github.com/nikwithak/tailwag/issues/new) or
[send me an email](mailto:nik@nikgilmore.com)!

##### Can I hire you to build or support my application?

Yes! Whether you are building with Tailwag or another stack, (Python,
TypeScript, React, Rust), send me an email me with the details of your project
at [nwakg@pm.me](mailto:nwakg@pm.me) and I will get back to you shortly.
