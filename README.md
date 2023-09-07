# Tailwag - The Rapid Rust Toolkit

This repo consolidates a variety of Rust libraries intended to rapidly create
scalable, Production-ready applications. The primary goal of Tailwag is to
enable same-day zero-to-production development and deployments.

## Project Status: WIP

This project is being actively developed in my free time.

### Production Readiness

There are known bugs that make this framework unsafe for Production use at this time.
While you are free to use this library for your own needs (see LICENSE), this library comes with no guarantees or warranties.

## Support / Troubleshooting

If you encounter any issues, please use GitHub Issues to report and track any bugs or feature requests.

## Contributions

If you like tailwag and would like to donate to the project, your donations are welcome and appreciated! Please message me privately to learn how.
I am not actively seeking pull requests or code contributions at this time. You are welcome to submit Pull Requests, and acceptance will be considered on a case-by-case basis.

## Quick-Start

// TODO: These commands don't work yet, but are the goal of how to init a new
application from scratch. Building towards this.

```bash
$> cargo install tailwag --features all

# To create a new web service
$> tailwag new --web-service

# To use a custom template (/path/to/template can be local or a github remote)
$> tailwag new --template </path/to/template>

# To deploy the application
$> tailwag deploy [-c ./tailwag.toml]


# Managing multiple applications in a monorepo, OR split across services



```

## Tenets [Opinionated Framework]

- [ ] The data structure is the most important piece of an application.

### Roadmap

- [ ] #[magic] macros that will give sensible defaults given only a basic struct.
- [ ]
- [ ]
- [ ]
- [ ]

## Goals

### Zero-to-Production in a Day

- [ ] Out-of-the-Box Production-Ready Prototypes
- [ ] Be able to deploy a Protoype into Production immediately, using preset stack configurations.
  - [ ] AWS Load-Balanced Service deployments
  - [ ] TLS certificates (self-signed or letsencrypt generated)
  - [ ] Automated deploy scripts

### Minimal Configuration & Sensible Defaults

- [ ] Have a working, Production-Ready REST API with nothing more than defining a struct, and a minimal amount of boilerplate.
- [ ] Allow enabling policy-based access controls, with a robust and extensible user permissions model out of the box.

### Minimal Boilerplate

- [ ] Write the logic once, and meet all mostly-universal use cases
  - [ ] Generate a Web API
  - [ ] Generate the object models and data manageres
  - [ ] Generate a front-end web management application (admin page)
    - NOTE: One of the issues I have with Django's default admin page is that it uses different

### Easy Overrides

- [ ] Modify or replace the standard implementation
- [ ] Make the data struct the source of truth for ALL related applications / dependencies.
- [ ] Allow multiple choices for data backend and caching strategies.

## Non-Goals

## Features

### FAQ

##### Q: Why Rust?

My experience has taught me that the most important part of building scalable, maintainable applications is to
clearly define the data types you'll be working with. One of the biggest traps I repeatedly see at startups is
the push to move fast instead of building well.

I find Rust to be such a joy to code in - the compiler checks my assumptions
and forces me to handle errors and edge cases. The memory safety and type
safety that comes along with Rust removes the need to worry about those things
on my own. I've heard it said from more than a few sources that "it's hard to
write bad code in Rust".

I have primarily a Java and Python background, and I find Rust to be the
perfect middle-ground between the two worlds - the type safety and compiler
guarantees of Java (and then some!), but without all the verbosity.

##### Q: Why did you write these from scratch instead of using existing crates

like `diesel` or `yew` or `<other_library>`?

A: I started these libraries as a learning experience, to teach myself the Rust
macro system and practice building reusable libraries.
