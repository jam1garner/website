<!--timestamp:1633576556-->

# Learning Autoref Specialization by Using it in a Real World Library

![](/img/owo-gameboy-colors.png)

Specialization is just one of *Those* kinds of Rust features—people who pay enough attention likely already know what I mean—those features that you remember hearing they were 'unstable' in some thread from when you started learning Rust and now a year or two or five later—only to check back in and feel like nothing has progressed. A feature doomed to sit in the dark recesses of the nightly compiler, slowly growing mold and oozing code-smell until the day a brave code janitor comes and brings it out of its misery.

Most of the times these nightly veterans *are* actually making lots of progress, even if it doesn't outwardly appear like it. A lot of this progress tends to come in the form of long-term work on [chalk], [polonius], and other efforts towards large-scale improvements of rustc internals.

[chalk]: https://github.com/rust-lang/chalk
[polonius]: https://github.com/rust-lang/polonius

In the case of specialization, however, things are... [less than stellar looking]. Now I'm no expert on language design, but one can deduce that "soundness hole" in all bold from the last major edit, a year and a half ago, is not the best sign for being able to use this feature soon.

[less than stellar looking]: https://github.com/rust-lang/rust/issues/31844

### What even is 'specialization'?

I'll try to keep this part quick, but for starters one feature of Rust's trait system is "blanket implementations". What that means is we can implement a trait for all types which implement another trait:

<script src="https://gist.github.com/jam1garner/68346f8fc834f121341e2812eeb1f858.js"></script>

the above implementation implements the `ToString` trait for all types which implement `Display`. That is to say, it defines that we can convert anything which we can display as text into a string. And this is great! This saves a lot of time implementing conversion to a string for a bunch of different types.

However this creates a problem. What if I have a type which has a *really* inefficient `Display` implementation, but its `ToString` implementation could be implemented more efficiently? If I try to implement `ToString` for my type, it'll cause an error:

```
error[E0119]: conflicting implementations of trait `std::string::ToString` for type `MyType`
  --> src/main.rs:14:1
   |
14 | impl ToString for MyType {
   | ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: conflicting implementation in crate `alloc`:
           - impl<T> ToString for T
             where T: std::fmt::Display, T: ?Sized;
```

Oh no! Now I either need to pick expressiveness or efficiency. That sucks. While 'Display' having a different behavior than 'ToString' isn't normal, wanting to make more specific implementations for a specific type *is* quite common.

The solution to this is 'specialization', a feature which lets you have both a blanket implementation and a 'specialized' implementation. A specialized implementation is one that implements a trait for a specific type such that it overrides the default/blanket implementation (simplfying for the sake of clarity, but that's the gist).

Now in order to understand how I came to need specialization myself, we need to get a bit of an understanding of the crate I needed it for, `owo-colors`, and understand what lead me to such a design in the first place. Once we cover that I hope it will be clear why specialization is useful here and serve as a concrete example from which we can cover how we can emulate it.

### What even is 'owo-colors'?

Well, [`owo-colors`] is my most popular library, although the idea isn't anything to write home about, it's just your run-of-the-mill terminal colors library. Honestly the interface isn't even original, it even tells you in the README that it's a [`colored`] rip-off:

[`owo-colors`]: https://crates.io/crates/owo-colors
[`colored`]: https://crates.io/crates/colored

> owo-colors is also more-or-less a drop-in replacement for colored

So then why does it even exist? Well for the most part the driving usecase was wanting to iterate on the design, with two of the motivating usecases being "displaying hex numbers" and "use on a `no_std` target", neither of which `colored` really supports. Despite that it was overkill for my specific target I decided to extend the goals a bit further by making it work on systems without an allocator.

For those who aren't familiar with `colored`'s interface, here's a code snippet from their README:

<script src="https://gist.github.com/jam1garner/26588049f4575c781bfb386a6c046647.js"></script>

the idea is pretty simple: to apply a color to a string, just use a method for the given color. The way this works is using an ["extension trait"](https://rust-lang.github.io/rfcs/0445-extension-trait-conventions.html), which is the design pattern of a trait you can import in order to add new methods to a specific existing type. This is often used for adding methods to types in the standard library which otherwise can't be augmented.

In this case, `colored` extends the `&str` and `String` types, adding methods which converts them to a `ColoredString`, which is basically just a `String` alongside foreground color, background color, and any effects (strikethrough, bold, etc) that should be applied.

This means in order to display a number in red we need to convert the number to a string, which means we need to allocate this new string. For `owo-colors`' goals this won't work, so we'll need to rethink the design.

### How `owo-colors` Works

First off let's cover the goals and what their implication is, in order to better understand the design:

1. We want to keep the `colored` interface - this means our core API will involve an extension trait which will have methods such as `.red()`, the output of which we should be able to display using [`fmt::Display`], the trait powering all Rust's formatting utilities.

2. We want to be able to display non-strings without allocating - this means we can't afford to store a String, and so we need to store a reference to our object to be displayed.

3. We want to be able to use formatters other than the [`fmt::Display`] trait (example: [`fmt::LowerHex`], so that we can display hexademical numbers) - this one is 'easy'. Since we already keep a reference in #2 we can just use whatever formatter we'd like.


Now putting this all together, this means we'll want something that looks something like...

<script src="https://gist.github.com/jam1garner/77fe0a96c7b594f65d5bac4a4ead6582.js"></script>

Now `owo-colors`'s definition is a bit more generic, but I'm omitting that for the sake of clarity. Now let's take a look at how this works in practice:

<script src="https://gist.github.com/jam1garner/8a9e9a88fdd36b7a890ac1ad4a526e94.js"></script>

Now let's think about how rustc looks at this: the `{:x}` means that the underlined portion needs to implement [`fmt::LowerHex`]. But what is the type of the underlined portion? Well `3.red()` is going to return `RedWrapper<&i32>`. Then we do `.on_blue()` and it returns `OnBlueWrapper<&RedWrapper<&i32>>`, so that's our type which needs to implement `fmt::LowerHex`.

So what we've gathered is: we want `OnBlueWrapper<&RedWrapper<&T>>` to implement [`fmt::LowerHex`] if `T` implements [`fmt::LowerHex`]. So the easiest way to do that is to implement:

1. `impl<T> fmt::LowerHex for RedWrapper<&T> where T: fmt::LowerHex`
2. `impl<T> fmt::LowerHex for OnBlueWrapper<&T> where T: fmt::LowerHex`

which will mean that so long as our inner type (in this case `i32`) implements our formatter trait (in this case [`fmt::LowerHex`]), so will our wrapper type. This means that any amount of chaining foreground and background colors will just add more wrapper types which modularly fit together in order to let you use any type with any colors.

### How `owo-colors` (Sometimes) Doesn't Work

Now for those of you out there who know how terminal colors work might've already pieced together: this design has a flaw. The first 18 releases of `owo-colors` all have this flaw. Well jokes on the people who know enough about ANSI escape sequences to try and guess the flaw: there's actually TWO flaws, so ha!

#### Issue #1: Temporaries

The first issue is quite simple: if you remember above our type was `OnBlueWrapper<&RedWrapper<...>>`, however one problem is: if we have `&RedWrapper<...>`, but not variable of type `RedWrapper`, that must mean the `&RedWrapper<...>` is a reference to a [temporary value](https://doc.rust-lang.org/reference/expressions.html#temporaries). Which means that if we try to store our `3.red().on_blue()` to a variable:

<script src="https://gist.github.com/jam1garner/4de5d4074fbce82951707f77fac36866.js"></script>

Rust gives us a nasty error:

```
error[E0716]: temporary value dropped while borrowed
 --> src/main.rs:4:21
  |
4 |     let red_three = 3.red().on_blue();
  |                     ^^^^^^^          - temporary value is freed at the end of this statement
  |                     |
  |                     creates a temporary which is freed while still in use
5 |     println!("{}", red_three);
  |                    --------- borrow later used here
  |
  = note: consider using a `let` binding to create a longer lived value
```

However, like it says, if we add another let binding:

<script src="https://gist.github.com/jam1garner/78d36f16b534abea58422ad2338d8556.js"></script>

then our program works as expected. So it's not *hard* for our users to work around this, but let's not pretend this style of usage is actually nice.

#### Issue #2: Output

To illustrate the other issue, let's make our own more-readable version of ANSI escape sequences that uses an XML syntax. It'll look something like this:

<script src="https://gist.github.com/jam1garner/0e0b4e819d572eaebba4d7b606988a63.js"></script>

so we have one sequence of characters which tells us where we start coloring (and which color): `<ansi red>` and we also have a sequence which tells us to stop: `</ansi>`. Ignoring a swap from non-printable characters to XML, this is effectively how terminal colors work.

Now, through this lens we can consider what our `OnBlueWrapper<&RedWrapper<&i32>>` display method's output looks like:

<script src="https://gist.github.com/jam1garner/de4573c950343693917496e14eb343cc.js"></script>

So breaking that down based on which type's [`fmt::LowerHex`] implementation wrote what:

|       Text       |      Written by      |
|------------------|----------------------|
| `<ansi on_blue>` | `OnBlueWrapper<...>` |
|   `<ansi red>`   |   `RedWrapper<...>`  |
|       `3`        |       `i32`          |
|    `</ansi> `    |   `RedWrapper<...>`  |
|    `</ansi>`     | `OnBlueWrapper<...>` |

However the problem here is that ANSI escape sequences allow multiple effects to be applied in a single sequence. Meaning the more efficient way to display the above looks like:

<script src="https://gist.github.com/jam1garner/48f49bfb97fe8e9eb87d6130a1713689.js"></script>

This optimization reduces the size of our escape sequences a good bit for heavily styled code, but there's a bit of a problem. In order to make such an optimization we need to have special behavior for when wrapper types are nested. And as we discussed back at the beginning of the article... Rust won't let us special-case the behavior of certain implementations of a trait.

So we need specialization, or our library will have to pick a new design. And since specialization is nightly only, guess we're done here. Thanks for reading the blog post! Have a nice rest of your day! Nothing left to talk about! There's no way we can solve our problem!

### How We Can Solve Our Problem

Alright, alright, I get it. You can't be fooled. You know I've got the good stuff in the back and I'm just holding out on you. Well fine, I'll go grab it, but it's going to cost you.

Now let's take a look at how method resolution works in Rust.

<script src="https://gist.github.com/jam1garner/4ca7e1c13a79f57c502e0d5b11d76c94.js"></script>

Now this isn't a "how well do you know Rust" quiz so I'll just show you what this prints:

```
printing for MyType
```

As for the "why"—when you run a method in Rust it has the ability to either dereference or borrow your variable until it finds a suitable method. The specific rules for this [are explained in the Rust reference](https://doc.rust-lang.org/reference/expressions/method-call-expr.html), but the general idea is that less dereferences/references means higher priority.

So in the above code snippet, `MyType::print` takes priority over `<MyType as Printable>::print` because unlike the trait, it takes `self`, not `&self`. This means no reference needs to be taken when calling the method, so it takes priority over the trait.

This means in certain scenarios (namely ones where the type is known and not generic) we can overrided a trait implementation for a specific type.

### Tying This All Together

Now that we have a mechanism for specialization, where do we apply it? Well, due to the fact we're abusing autoreference rules, it needs to be done at a method call. And since our only method calls in the `owo-colors` API are those of [`OwoColorize`], that means those are the only things we can specialize (meaning: we cannot specialize our `fmt::LowerHex` implementation itself).

Now knowing that, we only have one option: adding duplicate methods to `RedWrapper` which take `self` instead of `&self` in order to override behavior.

<script src="https://gist.github.com/jam1garner/547a67ad64f92fac4883bda2619b982e.js"></script>

then we can provide a combination foreground-and-background wrapper `RedOnBlue`, which can then implement our more efficient output. Also worth noting that since in the actual `owo-colors` the types are generic over the color (for example `RedOnBlueWrapper` would actually be [`ComboColorDisplay<'_, Red, Blue, T>`]), this is not actually as gross as it seems.

But wait! There's more. The autoref specialization has not just solved our efficiency problem. It also solved our ergonomics problem. Since our specialized methods must take `self`, that means we're not creating a reference to the original color wrapper. We instead extract the `&T` from it and create a new type. Which means no temporary variable when chaining methods.

Which means suddenly, in a single truly elegant (albeit entirely accidental) move, we have made it so we can chain methods to store for later, entirely solving both of the core issues on the library's design:

<script src="https://gist.github.com/jam1garner/f24a2a609f48e9ee400e587463298b8a.js"></script>

This means finally, after 18 versions and over 750k downloads, these core issues have finally been resolved. And alongside this blog post I have released `owo-colors` 3, and I'd appreciate it if you checked it out! It's already being used in great libraries by awesome people like [`color-eyre`], [`miette`], and a bunch of great CLI tools.

And thanks a ton to those working on [`nushell`] for giving me the last little nudge I needed to figure this out while they were evaluating using it.

[`fmt::Display`]: https://doc.rust-lang.org/std/fmt/trait.Display.html
[`fmt::LowerHex`]: https://doc.rust-lang.org/std/fmt/trait.LowerHex.html
[`OwoColorize`]: https://docs.rs/owo-colors/*/owo_colors/trait.OwoColorize.html
[`ComboColorDisplay<'_, Red, Blue, T>`]: https://docs.rs/owo-colors/*/owo_colors/struct.ComboColorDisplay.html

[`color-eyre`]: https://crates.io/crates/color-eyre
[`miette`]: https://crates.io/crates/miette
[`nushell`]: https://www.nushell.sh/
