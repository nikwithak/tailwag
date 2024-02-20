# Tailwag - The Rapid Rust Toolkit

This repo consolidates a variety of Rust libraries intended to rapidly create
scalable, Production-ready applications. The end goal of Tailwag is to enable
same-day zero-to-production development and deployments.

## Project Status: WIP

This project is being actively developed in my free time. Expect breaking
changes

## Support and Contributions

This

### Production Readiness

There are known bugs and security risks that make this framework unsafe for
Production use at this time.

## Quick-Start

// TODO: These commands don't work yet, but are the goal of how to init a new
application from scratch. Building towards this.

```bash
$> cargo install tailwag --features all

# To create a new web service
$> tailwag new --web-service <project_name>

# To use a custom template (/path/to/template can be local or a github remote)
$> tailwag new --from-template </path/to/template> <project_name>

# To run the application locally in debug mode.
$> tailwag debug <project_name>

# To deploy the application
$> tailwag deploy [-c ./tailwag.toml]

# Managing multiple applications in a monorepo, OR split across services
```

## Tenets [Opinionated Framework]

- [ ] The data model is the most important part of an application, and therefore
      it's essential to make it consistent and easy to modify. This framework
      seeks to make it simple and fast to define the core data structures needed
      for an application, and handle the standard infrastructure patterns needed
      to support it.
- [ ] Focus on Ergonomics - Tailwag enables you to define all aspects of a data
      type from its definition. Custom overrides of functionality should be easy
      for customization. Validation, when possible. Rely heavily on traits and
      derive macros. Zero effort for the default case, but easy to override
      behavior.
- [ ] Define things in terms of Data. Data Types have a Data Source model and a
      Data Output model. Sources I want to support:
  - REST API
  - User Input (web forms)
  - Triggers (ASYNC queued tasks)
  - Triggers (SYNC webhook)
- [ ] All data types should have a customizable access policy. With policies
      enabled, data will be filtered automatically, with sensible defaults.

### Roadmap

- [ ] #[magic] macros that will give sensible defaults given only a basic
      struct. Per the above tenet to favor derive macros and traits,
      the #[magic] macro will primarily just add a bunch of derive macros.
- [ ]
- [ ]
- [ ]
- [ ]

## Goals

### Zero-to-Production in a Day

- [ ] Out-of-the-Box Production-Ready Prototypes
- [ ] Be able to deploy a Protoype into Production immediately, using preset
      stack configurations.
  - [ ] AWS Load-Balanced Service deployments
  - [ ] TLS certificates (self-signed or letsencrypt generated)
  - [ ] Automated deploy scripts

### Minimal Configuration & Sensible Defaults

- [ ] Have a working, Production-Ready REST API with nothing more than defining
      a struct, and a minimal amount of boilerplate.
- [ ] Allow enabling policy-based access controls, with a robust and extensible
      user permissions model out of the box.

### Minimal Boilerplate

- [ ] Write the logic once, and meet all mostly-universal use cases
  - [ ] Generate a Web API
  - [ ] Generate the object models and data manageres
  - [ ] Generate a front-end web management application (admin page)
    - NOTE: One of the issues I have with Django's default admin page is that it
      uses different

### Easy Overrides

- [ ] Modify or replace the standard implementation
- [ ] Make the data struct the source of truth for ALL related applications /
      dependencies.
- [ ] Allow multiple choices for data backend and caching strategies.

## Non-Goals

## Features

### FAQ

##### Q: Why Rust?

My experience has taught me that the most important part of building scalable,
maintainable applications is to clearly define the data types you'll be working
with. One of the biggest traps I repeatedly see at startups is the push to move
fast instead of building well.

I find Rust to be such a joy to code in - the compiler checks my assumptions and
forces me to handle errors and edge cases. The memory safety and type safety
that comes along with Rust removes the need to worry about those things on my
own. I've heard it said from more than a few sources that "it's hard to write
bad code in Rust".

I have primarily a Java and Python background, and I find Rust to be the perfect
middle-ground between the two worlds - the type safety and compiler guarantees
of Java (and then some!), but without all the verbosity.

##### Q: Why not use existing frameworks like `diesel` or `yew` or `<other_library>`?

A: I started these libraries as a learning experience, and it has grown over
time into something more useful than that.
