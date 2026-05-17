# Tailwag ORM

This crate is the ORM used by
[the Tailwag web framework](https://github.com/nikwithak/tailwag). It implements
a Manager pattern, and contains a suite of data structs for interfacing with /
representing Postgres transactions.

Also included is a set of macros for deriving the data structures, Postgres
query builders, and filterable types for a struct with minimal boilerplate.

Status: Experimental. Future versions may contain breaking API changes.

Currently supports structs made up of primitive types, filtering on primitive
types, and one-to-one parent/child relationships.

For more examples, and to see how it integrates with the larger Tailwag web
framework, check out the `[tailwag](https://github.com/nikwithak/tailwag)`
repository.

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
