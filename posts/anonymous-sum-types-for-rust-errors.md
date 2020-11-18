<!--timestamp:1605671528-->

# Anonymous Sum Types for Rust Error Handling

![](/img/sum_type_result.png)

Anonymous Sum Types are one of those Rust features that doesn't exist yet and you either are very familiar with the idea and can't wait for the day an RFC gets approved or... you've never heard of it and think the name sounds as gibberish. Luckily in my experience the transition from the latter to the former is a smooth one!

> [docs.rs link for those who just want to skip to a code example](https://docs.rs/some-error)

![](https://media.discordapp.net/attachments/376971848555954187/778499930305986590/unknown.png)

## What's an Anonymous Sum Type?

Let's break it down word for word. (If you're already familiar, feel free to skip this)

> Anonymous

Anonymity, the programming term, refers to a variant of a construct which bears no name. For example, in Javascript you have this little guy:

<script src="https://gist.github.com/jam1garner/3b292e7f6f213143fc114c77bb80dcac.js"></script>

And, for those who are neither familiar with javascript nor anonymous functions as a whole, you might be confused about what the purpose of a function with no name to call it with! However a function in javascript is an object, which means the above is an expression. So any API which takes a function as a parameter can now take an anonymous function. So you can do the following:

<script src="https://gist.github.com/jam1garner/18fe896e800a0c94f05dab8ddddf49db.js"></script>

And if you're familiar with Rust, you'll realize Rust has a very similar construct, closures!

> Sum Types

I'm sure if you're reading this you're likely familiar with both a sum (as most programmers are familiar with algebra) and a type, but what exactly is a sum type? (also known as a "tagged union" or "discriminated union" or "tagged variant" and probably one or two other things I'm sure)

You might have even used a sum type without being familiar with the overly academic nomenclature of "sum type", let's take at a really common use case in Rust for sum types and, "coincidentally", the topic of this blog post: error handling.

In Rust error handling it has a strong emphasis on returning errors, as opposed to many languages which reach for exceptions. Here is how this difference looks in *reality*:


<script src="https://gist.github.com/jam1garner/044b5af8ceba15642197359accf382c8.js"></script>

<script src="https://gist.github.com/jam1garner/cd0ea506305a5baa5532450c0866273c.js"></script>

In the Java/exceptions version, the addition of error handling does not affect the return type. However, in Rust, you return one of the two types: either `CsvFile` or `io::Error`. So, if you think about the actual type you are returning, `Result<CsvFile, io::Error>` is a type representing the combination of the two types. (Spoiler: this is what a sum type is, more on that in a second)

Let's try doing something similar by hand in Rust:

<script src="https://gist.github.com/jam1garner/a2e3232f8643052c9cbd0e8c77a37e50.js"></script>

Now, at risk of over-explaining the concept, let's represent this in a different manner. A type is merely a set of values that allows us to constrain the values we have to work with. Let's try representing `bool` and `u32` as sets:

```
bool = { true, false }
u32 = { 0, 1, 2, 3, ... , (2^32) - 3, (2^32) - 2, (2^32) - 1 }
```

that means we can represent our type `BoolOrU32` as the union of the set `bool` and the set `u32`:

```
BoolOrU32 = bool ∪ u32 = { true, false, 0, 1, 2, 3, ... , (2^32) - 3, (2^32) - 2, (2^32) - 1 }
```

Now one thing to consider is that with our `BoolOrU32` enum, we are still storing the information which tells us if it is a `bool` or a `u32`. This is what differentiates a sum type from a C-styled union. This concept of storing which variant the enum is is the "tagged" part of the other name for sum types, "tagged unions".

To put that all together, **an anonymous sum type is an unnamed combination of types**.

## Error Handling Woes

Taking a look back at our previous example there's a bit of a problem:

```
warning: this is super limiting
  --> ~/dev/csv-parser/src/lib.rs
   |
33 | fn parse_csv(path: &Path) -> Result<CsvFile, io::Error>;
   |                                              ^^^^^^^^^ this sucks
   |
   = help: consider using one of the following libraries:
           thiserror
           eyre
           snafu
```

While some errors from parsing a file might be I/O related (for example if the file doesn't exist), but some are parsing related errors! We could use `io::Error::new(io::ErrorKind::Other, my_error_here)` but that's rather verbose to construct, hacky, and it's even somehow an order worse to extract our error type from the `io::Error`!

One solution to this problem is to hand roll an `enum` to make a sum type for your error type. This is a really great way of going about things, and is my goto for error handling! It's a great fit for libraries as it allows library consumers to handle only certain errors, or specify error handling by type. It's not very helpful to handle a missing file and an invalid webpage certificate the same way!

However... this introduces a new problem. Internally, you often want granularity as far as your error types. It allows people new to the codebase to see exactly which 2-3 errors a function could return, it allows you to use exhaustive matching to catch more bugs at compile time, etc. However, it's not very appealing to make a new enum for every new combination of errors a function produces. I mean how do you even get past the gruelling process of naming such a type? `IoOrParseError` isn't exactly what I call a lovely name...

## A Solution! (Sort of)

Not wanting to name a sum type? That sounds like a job for... anonymous sum types! This is, admittedly, one of the features I've wanted since I first fell in love with Rust's enums being sum types (in a systems programming language? scandelous!). However, last I checked, bumping every Sum Type RFC with an alt got me nowhere! (Yes, for anyone who is paying attention, I *will* recycle the exact same joke as from my RustConf talk)

It's time to take matters into my own hands. Introducing `some-error`!

<script src="https://gist.github.com/jam1garner/799ef48a78599f0c82494bd5a07e2def.js"></script>

The idea is simple: use an attribute macro to allow for a very minimal/focused version of anonymous sum types specifically aimed at use in error handling. To use it you just slap `#[some_error]` on your function, set the return type as `Result<T, E1 + E2 + ...>` and you're good to go! You can even use `?` for any of the included error types.

> "Wait, what's the big idea with this hand-wavey black magic nonsense!"

## The Man Behind The Curtain

### The Syntax

In Rust, you can return trait objects, for example:

<script src="https://gist.github.com/jam1garner/914650535a6967ab9a78fd1e09723c46.js"></script>

Where `dyn Any + Send + Sync` can be any object which can be any type that implements `Any`, `Send`, and `Sync`, the same single function can even return any number of types, so long as all of those types implement `Any + Send + Sync`. Tiny bit of [Rust history](https://doc.rust-lang.org/edition-guide/rust-2018/trait-system/dyn-trait-for-trait-objects.html), but before Rust 2018, the syntax for this didn't include `dyn`:

<script src="https://gist.github.com/jam1garner/6c91d959ede9e9fb9e8db635eefd942d.js"></script>

And since this still parses in spite of being deprecated, that means we can use it for our own nefarious macro purposes! (~~until it is inevitably removed~~)

### The Pattern Matching

Unlike the function itself, matching on the anonymous sum type requires no macro. That means this time around we can't cheat! Everything must be above board or `rustc` will give me a slap on the wrist. Although, I can't quite say I got out of the macro hackery unscathed, [`rustc` punished me with a bit of ICE for my crimes](https://github.com/rust-lang/rust/issues/79148).

This time, I abuse a lesser-known [Rust fact](/img/rust-fact.png) that Rust functions and modules exist in different scopes, allowing you to make them overlapping:

<script src="https://gist.github.com/jam1garner/e9841edfba475ba622431b6912910d80.js"></script>

And, similarly to traits and derive macros, this allows you to import both at once:

<script src="https://gist.github.com/jam1garner/53eb0f3a576010a0b56ab9cfe14b2257.js"></script>

which gives the illusion that the function can have associated constants, types, functions, etc.

### Making `?` "Just Work"

This part is rather easy! One really wonderful feature of Rust is that the `?` operator will automatically coerce from `Result<T, E1>` to `Result<T, E2>`, given `E2: From<E1>`. Meaning all we have to do is generate some trivial `From` implementations and voila! Now any part of the sum type can be returned using `?`.

### The Variants

One problem that occurred while making this is that, unlike a normal enum, the variants can't really have identifiers as names. Since it's a common pattern in Rust to refer to errors in the form of `module::Error` (such as `io::Error` or `fmt::Error`), the variants need to be paths (as I don't really consider any form of normalization such as `IoError` to really be an appealing solution).

So how can we go about this? Since we're already using a macro anyways, we might as well make the most of it! Let's just generate modules to emulate the paths and then re-export the individual variants of our error type!

<script src="https://gist.github.com/jam1garner/a0b3e9291e55b18a01975ab2001d0f22.js"></script>

## Wait What Do You Mean "Sort of" a Solution?

Welllllllllllllll. While I am rather happy with how far I was capable of pushing what is possible, I can't say whether or not I actually like it. I haven't been able to extensively test it as I started writing the library a day before writing this. Perhaps this results in a lot of problems! Maybe those problems are all solvable by language-level support of anonymous sum types. Maybe someone else needs to "yes and" this idea before it is actually truely useful. I can't say!

My goal here was not to make **The** error handling library, but rather to explore a new design idea. And possibly to give people something to play with to find sharp edges to motivate a language-level implementation.

You're free to use some-error in your projects, and I will certainly maintain it to the best of my ability, but I guess what I'm trying to say is please don't flame me on reddit just because this library isn't going in anyone's million LOC codebase.

**Related Links:**

[Documentation](https://docs.rs/some-error)

[Source](https://github.com/jam1garner/some-error)

[My Twitter](https://twitter.com/jam1garner)

[Reddit Discussion](https://www.reddit.com/r/rust/comments/jwnsp4/anonymous_sum_types_for_rust_error_handling)
