# Tailwag Macros

This crate is a collection of macros to support the
[`tailwag`](https://github.com/nikwithak/tailwag) framework.

Worth noting is that there are some obtuse patterns in use here (like using so
many crates... which was overkill) that I thought would make things simpler
long-term, but they just made them more confusing. I've been slowly breaking
macros OUT of this crate, and moving the macros underneath the specific crates
they support.
