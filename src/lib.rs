#![no_std]
//! # Lifetime Abstractions
//!
//! A _lifetime abstraction_ is a type with a bound placeholder lifetime that can later be replaced
//! with a different lifetime. As such they can be seen as a type-level function from lifetimes to
//! types.
//!
//! For example [`Lt!(for<'a> &'a str)`][Lt!] is the type of a string slice having the placeholder
//! as lifetime. Viewed from the outside, the abstraction type hides the bound placeholder lifetime.
//!
//! We can see that it has no unbound lifetimes by defining a type alias with no lifetime
//! parameters:
//!
//! ```rust
//! # use lifetime_abstractions::*;
//! type NoLifetimeParameters = Lt!(for<'a> &'a str);
//! ```
//!
//! Using the [`LtApply`] type, we can obtain the inner type from an abstraction with another
//! lifetime substituted for the bound placeholder lifetime:
//!
//! ```rust
//! # use lifetime_abstractions::*;
//! # type NoLifetimeParameters = Lt!(for<'a> &'a str);
//! const STATIC_STR: LtApply<'static, NoLifetimeParameters> = "Hello, world!";
//!
//! fn borrow_str<'a>(string: &'a String) -> LtApply<'a, NoLifetimeParameters> {
//!     &string
//! }
//! ```
//!
//! We can also pass a lifetime abstraction as a type parameter. In that case we need to add the
//! [`LtAbs`] trait bound before we can use [`LtApply`] on the generic lifetime abstraction:
//!
//! ```rust
//! # use lifetime_abstractions::*;
//! # type NoLifetimeParameters = Lt!(for<'a> &'a str);
//! struct Static<T: LtAbs>(LtApply<'static, T>);
//! struct Borrowed<'a, T: LtAbs>(LtApply<'a, T>);
//!
//! const STATIC: Static<NoLifetimeParameters> = Static("Hello, world!");
//!
//! fn borrowed<'a>(string: &'a String) -> Borrowed<'a, NoLifetimeParameters> {
//!     Borrowed(string.as_str())
//! }
//! ```
//!
//! Using lifetime abstractions as associated types allows us to emulate restricted form of _generic
//! associated types_ (GATs). The restriction being that all parameters for the associated type are
//! lifetimes.
//!
//! ## Example: Streaming Iterators
//!
//! The classic example where GATs would be useful are _streaming iterators_. These are iterators
//! where the returned item is allowed to borrow from the iterator itself, i.e. where calling next
//! may invalidate the previous item. With GATs we could do this in the following way:
//!
//! ```compile_fail
//! pub trait StreamingIterator {
//!     type Item<'a>;
//!
//!     fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
//! }
//!
//! struct Countdown {
//!     buf: String,
//!     count: usize,
//! }
//!
//! impl StreamingIterator for Countdown {
//!     type Item<'a> = &'a str;
//!
//!     fn next<'a>(&'a mut self) -> Option<&'a str> {
//!         if self.count == 0 {
//!             return None;
//!         }
//!         self.count -= 1;
//!         self.buf.clear();
//!         write!(&mut self.buf, "{}", self.count).unwrap();
//!         Some(&self.buf)
//!     }
//! }
//!
//! ```
//!
//! Using lifetime abstractions we can implement streaming iterators in a very similar way on stable
//! Rust:
//!
//! ```rust
//! # use lifetime_abstractions::*;
//! # use std::fmt::Write;
//! pub trait StreamingIterator {
//!     type Item: LtAbs;
//!
//!     fn next<'a>(&'a mut self) -> Option<LtApply<'a, Self::Item>>;
//! }
//!
//! struct Countdown {
//!     buf: String,
//!     count: usize,
//! }
//!
//! impl StreamingIterator for Countdown {
//!     type Item = Lt!(for<'a> &'a str);
//!
//!     fn next<'a>(&'a mut self) -> Option<&'a str> {
//!         if self.count == 0 {
//!             return None;
//!         }
//!         self.count -= 1;
//!         self.buf.clear();
//!         write!(&mut self.buf, "{}", self.count).unwrap();
//!         Some(&self.buf)
//!     }
//! }
//! ```
//!
//! ## Lifetime Elision
//!
//! Lifetime abstractions support elision of lifetimes. The placeholder lifetime will be assigned to
//! all elided lifetimes, e.g. [`Lt!(&str)`][Lt!] is the same as [`Lt!(for<'a> &'a str)`][Lt!]. In
//! particular, all examples above could be written without mentioning any lifetimes.
//!
//! The same lifetime abstraction can combine elided lifetimes with lifetimes bound outside of the
//! abstraction, e.g:
//!
//! ```rust
//! # use lifetime_abstractions::*;
//! type Elided<'outer> = Lt!(&[&'outer str]);
//! type Expanded<'outer> = Lt!(for<'a> &'a [&'outer str]);
//! ```
//!
//! ## Implementation
//!
//! Lifetime abstractions `Lt!(for<'a> Something<'a>)` are represented using [function pointer
//! types][`fn`] of the form `for<'a> fn(Lt<'a>) -> Something<'a>`.
//!
//! The [`Lt<'a>`][struct@Lt] type's only role is to ensures that the lifetime is invariant and that
//! it appears in argument position. It would be possible to use `&'a mut ()` instead. Using a
//! custom type has the advantage that error messages will point to this crate.
//!
//! Instead of function pointers, it would also be possible to use trait objects of a custom trait.
//! `for<'a> dyn AbsTrait<Lt<'a>, Output=Something<'a>>`. One downside of this is that it breaks
//! lifetime elision within abstractions. Another is that trait objects are unsized, so this would
//! either require `?Sized` bounds in user code or some additional indirection. Finally, while for
//! writing type abstractions we could hide this more verbose syntax behind the [`Lt!`] macro, it
//! would still appear in error messages.
//!
//! While the special syntax for function pointer types makes this much more readable, the downside
//! is that there is no direct type-level way to get the output type of a function pointer given
//! only the argument type. This is important as it is exactly what is needed to implement
//! [`LtApply<'a, T>`][LtApply], which is the return type of the function pointer `T` for the
//! argument `Lt<'a>`.
//!
//! With some helper traits in [`fn_helpers`] this can be worked around.
//! [`FnOutput1`][`fn_helpers::FnOutput1`] makes the output type available given only the argument
//! type. [`FnBound1`][`fn_helpers::FnBound1`] asserts that this indeed matches the output type of
//! the function pointer. The latter is needed to avoid "implementation is not general enough"
//! errors when [HRTBs] are involved.
//!
//! Finally, the `LtAbs` trait has the bound `for<'a> FnOutput1<Lt<'a>>` together with a blanket
//! impl for all such types.
//!
//! [HRTBs]:https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds
//!
//! ## MSRV
//!
//! This implementation is compatible with `rustc 1.46.0` and newer. On older versions checking of
//! the trait bounds on [`FnBound1`][`fn_helpers::FnBound1`] fails. The alternative implementation
//! using trait objects, mentioned above, seems to work down to `rustc 1.17.0`. Given the downsides
//! of that alternative also mentioned above, I do not plan to support versions older than `rustc
//! 1.46.0`.
//!
//! ## Alternatives, Prior Art and Limitations
//!
//! I haven't seen this exact technique before, it would have saved me quite some time if I did, but
//! of course I haven't looked everywhere. If this approach was already documented anywhere before,
//! please let me know so I can mention it here.
//!
//! As far as I am aware, all previous alternatives (for emulating lifetime parameterized GATs as
//! well as some other use cases I have in mind) have some downsides compared to this. They require
//! introducing additional lifetime parameters to traits and/or implementations, defining new helper
//! types or traits for each use and/or have limiting `'static` bounds in some places.
//!
//! Lukas Kalbertodt's article ["Solving the Generalized Streaming Iterator Problem without
//! GATs"][streaming-iterator-article] has a nice overview of some of these alternatives and their
//! limitations.
//!
//! The only limitation of this approach I've run into so far is that it sometimes requires
//! additional type hints in places where I would expect type inference to be sufficient. This
//! happens quite often when closure types interact with lifetime abstractions and may require [this
//! technique to add a sufficiently generic type hint to a closure][constrain-closure].
//!
//! [streaming-iterator-article]:http://lukaskalbertodt.github.io/2018/08/03/solving-the-generalized-streaming-iterator-problem-without-gats.html
//! [constrain-closure]:https://stackoverflow.com/a/46198877

use core::marker::PhantomData;

/// Helper traits for type-level application of closures (and function pointers).
///
/// These can be freely used, but are not necessary when using the API exported from crate root.
pub mod fn_helpers {
    /// Helper trait used to recover the output type of a 1-argument closure given only the input
    /// type.
    ///
    /// For convenience, you can access the output type using the [`Apply1`] type alias.
    ///
    /// Using the unstable `unboxed_closures` feature we could use `FnOnce<(Arg,)>` instead, but
    /// currently on stable Rust we always have to use the `FnOnce(Arg) -> Output` syntax which
    /// requires us to constrain the output.
    pub trait FnOutput1<Arg> {
        /// The output type returned by the closure.
        type FnOutput1;
    }

    /// The output type returned when calling the 1-argument closure `Fn1` whith an argument of type
    /// `Arg`.
    pub type Apply1<Fn1, Arg> = <Fn1 as FnOutput1<Arg>>::FnOutput1;

    impl<T, Arg, Output> FnOutput1<Arg> for T
    where
        T: FnOnce(Arg) -> Output,
    {
        type FnOutput1 = T::Output;
    }

    /// Trait of 1-argument closures with an unconstrained output type.
    ///
    /// This has [`FnOutput1`] and [`FnOnce`] as supertypes. It uses [`FnOutput1`] to recover the
    /// output required for the [`FnOnce`] bound on stable Rust.
    pub trait FnBound1<Arg>:
        FnOutput1<Arg> + FnOnce(Arg) -> <Self as FnOutput1<Arg>>::FnOutput1
    {
    }

    impl<T, Arg> FnBound1<Arg> for T where
        T: FnOutput1<Arg> + FnOnce(Arg) -> <Self as FnOutput1<Arg>>::FnOutput1
    {
    }
}

use fn_helpers::*;

/// A lifetime binding. Instead of using this directly, preferably use [`Lt!`] and [`LtApply`].
///
/// See [`Lt!`] for how this type is used.
pub struct Lt<'a>(PhantomData<&'a mut ()>);

/// A lifetime abstraction.
pub trait LtAbs: for<'a> FnOutput1<Lt<'a>> {}

impl<T> LtAbs for T where T: for<'a> FnOutput1<Lt<'a>> {}

/// Substitutes a concrete lifetime for the bound lifetime in an abstraction.
pub type LtApply<'a, Abs> = Apply1<Abs, Lt<'a>>;

/// Creates a lifetime abstraction, binding a placeholder lifetime.
///
/// This allows writing `Lt!(for<'a> SomeType<&'a str>)` which will expand to `for<'a> fn(Lt<'a>) ->
/// SomeType<&'a str>`. It also supports lifetime elision where `Lt!(SomeType<&str>)` will expand to
/// `fn(Lt) -> SomeType<&str>`.
#[macro_export]
macro_rules! Lt {
    (for<$lt:lifetime> $ty:ty) => { for<$lt> fn($crate::Lt<$lt>) -> $ty };
    ($ty:ty) => { fn($crate::Lt) -> $ty };
}
