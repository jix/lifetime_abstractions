# Lifetime Abstractions

[![github][github-badge]][github]
[![crates.io][crate-badge]][crate]
[![docs.rs][docs-badge]][docs]

A _lifetime abstraction_ is a type with a bound placeholder lifetime that can later be replaced
with a different lifetime. As such they can be seen as a type-level function from lifetimes to
types.

For example `Lt!(for<'a> &'a str)` is the type of a string slice having the placeholder
as lifetime. Viewed from the outside, the abstraction type hides the bound placeholder lifetime.

We can see that it has no unbound lifetimes by defining a type alias with no lifetime
parameters:

```rust
type NoLifetimeParameters = Lt!(for<'a> &'a str);
```

Using the `LtApply` type, we can obtain the inner type from an abstraction with another
lifetime substituted for the bound placeholder lifetime:

```rust
const STATIC_STR: LtApply<'static, NoLifetimeParameters> = "Hello, world!";

fn borrow_str<'a>(string: &'a String) -> LtApply<'a, NoLifetimeParameters> {
    &string
}
```

We can also pass a lifetime abstraction as a type parameter. In that case we need to add the
`LtAbs` trait bound before we can use `LtApply` on the generic lifetime abstraction:

```rust
struct Static<T: LtAbs>(LtApply<'static, T>);
struct Borrowed<'a, T: LtAbs>(LtApply<'a, T>);

const STATIC: Static<NoLifetimeParameters> = Static("Hello, world!");

fn borrowed<'a>(string: &'a String) -> Borrowed<'a, NoLifetimeParameters> {
    Borrowed(string.as_str())
}
```

Using lifetime abstractions as associated types allows us to emulate restricted form of _generic
associated types_ (GATs). The restriction being that all parameters for the associated type are
lifetimes.

## Example: Streaming Iterators

The classic example where GATs would be useful are _streaming iterators_. These are iterators
where the returned item is allowed to borrow from the iterator itself, i.e. where calling next
may invalidate the previous item. With GATs we could do this in the following way:

```rust
pub trait StreamingIterator {
    type Item<'a>;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}

struct Countdown {
    buf: String,
    count: usize,
}

impl StreamingIterator for Countdown {
    type Item<'a> = &'a str;

    fn next<'a>(&'a mut self) -> Option<&'a str> {
        if self.count == 0 {
            return None;
        }
        self.count -= 1;
        self.buf.clear();
        write!(&mut self.buf, "{}", self.count).unwrap();
        Some(&self.buf)
    }
}

```

Using lifetime abstractions we can implement streaming iterators in a very similar way on stable
Rust:

```rust
pub trait StreamingIterator {
    type Item: LtAbs;

    fn next<'a>(&'a mut self) -> Option<LtApply<'a, Self::Item>>;
}

struct Countdown {
    buf: String,
    count: usize,
}

impl StreamingIterator for Countdown {
    type Item = Lt!(for<'a> &'a str);

    fn next<'a>(&'a mut self) -> Option<&'a str> {
        if self.count == 0 {
            return None;
        }
        self.count -= 1;
        self.buf.clear();
        write!(&mut self.buf, "{}", self.count).unwrap();
        Some(&self.buf)
    }
}
```

## Lifetime Elision

Lifetime abstractions support elision of lifetimes. The placeholder lifetime will be assigned to
all elided lifetimes, e.g. `Lt!(&str)` is the same as `Lt!(for<'a> &'a str)`. In
particular, all examples above could be written without mentioning any lifetimes.

The same lifetime abstraction can combine elided lifetimes with lifetimes bound outside of the
abstraction, e.g:

```rust
type Elided<'outer> = Lt!(&[&'outer str]);
type Expanded<'outer> = Lt!(for<'a> &'a [&'outer str]);
```

## Implementation

TODO rewrite

## MSRV

TODO retest

## Alternatives, Prior Art and Limitations

I haven't seen this exact technique before, it would have saved me quite some time if I did, but
of course I haven't looked everywhere. If this approach was already documented anywhere before,
please let me know so I can mention it here.

As far as I am aware, all previous alternatives (for emulating lifetime parameterized GATs as
well as some other use cases I have in mind) have some downsides compared to this. They require
introducing additional lifetime parameters to traits and/or implementations, defining new helper
types or traits for each use and/or have limiting `'static` bounds in some places.

Lukas Kalbertodt's article ["Solving the Generalized Streaming Iterator Problem without
GATs"][streaming-iterator-article] has a nice overview of some of these alternatives and their
limitations.

TODO mention the implicit fixed `Sized` bound

The only limitation of this approach I've run into so far is that it sometimes requires
additional type hints in places where I would expect type inference to be sufficient. This
happens quite often when closure types interact with lifetime abstractions and may require [this
technique to add a sufficiently generic type hint to a closure][constrain-closure].

[streaming-iterator-article]:http://lukaskalbertodt.github.io/2018/08/03/solving-the-generalized-streaming-iterator-problem-without-gats.html
[constrain-closure]:https://stackoverflow.com/a/46198877

## License

This software is available under the Zero-Clause BSD license, see
[LICENSE](LICENSE) for full licensing information.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this software by you shall be licensed as defined in
[LICENSE](LICENSE).

[github]:https://github.com/jix/lifetime_abstractions
[crate]:https://crates.io/crates/lifetime_abstractions
[docs]:https://docs.rs/lifetime_abstractions/*/lifetime_abstractions

[github-badge]: https://img.shields.io/badge/github-jix/lifetime_abstractions-blueviolet?style=flat-square
[crate-badge]: https://img.shields.io/crates/v/lifetime_abstractions?style=flat-square
[docs-badge]: https://img.shields.io/badge/docs.rs-lifetime_abstractions-informational?style=flat-square
