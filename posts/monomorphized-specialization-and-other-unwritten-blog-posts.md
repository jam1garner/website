<!--timestamp:1615722341-->

# Post-Monomorphization Dead Code Elimination and Other Unwritten Blog Posts

![](/img/feature_specialization.png)


So I've had a lot of ideas for blog posts lately, but none of them *quite* panned out. Rather than throw away what I hope is valuable insight, I'm instead taking them and combining them into one larger blog post that will hopefully be useful in spite of none of the individual parts being worth their own blogpost.

* [Post-Monomorphization Dead Code Elimination to Emulate Specialization](#specialization)
* [Building Rust Windows Installers in CI](#installers)
* [Measurement of Macro Test Coverage](#macro-coverage)
 

## <a name="specialization"></a> Post-Monomorphization Dead Code Elimination to Emulate Specialization

Rust's implementation of generics, like C++'s and others', relies heavy on monomorphization, aka compile-time copy/pasting. Since each instance of a generic function can rely on type-specific optimizations, each instance of the generic will be as efficient as possible for the given type. This also is beneficial as, unlike vtables, there isn't a layer of indirection involved in calling the generic implementation. (Note: Rust doesn't purely rely on monomorphization, it uses vtables for `dyn Trait` to allow additional flexibility when needed)

This is great for compiler optimizations, but not so much user-provided ones. Despite each instance *eventually* becoming its own function, there's no "manual override" for when the compiler itself couldn't possible know what optimizations would benefit that particular type. This is one of the many use-cases of "specialization", or allowing overlapping implementations of a trait, where the more-specific implementation takes precedence. It's a great feature! Just don't use it if you want to see the stable toolchain soon. Unfortunately it's a really tough problem to get right, so language support for specialization isn't quite here yet.

In the meantime though, it's not a terrible idea to use other methods of achieving similar results. One such trick is using Rust's dynamic typing system to implement a subset of specialization in a zero-cost manner to help squeeze out performance while still staying expressive. "Dynamic typing?" I can imagine some of you thinking. "But this is Rust!". And while Rust is primarily statically typed (with just the right amount of type inferrence to keep us all sane), it *does* have dynamic typing in the form of the [`Any`](https://doc.rust-lang.org/std/any/trait.Any.html) trait.

The `Any` trait gives us two abilities, downcasting and getting a unique id from a type:

<script src="https://gist.github.com/jam1garner/2aa4128edffa0426fa6d01b2aa2c8c00.js"></script>

Using this, we can have a generic which has unique behavior for a given type:

<script src="https://gist.github.com/jam1garner/c1df9d2435611e44c2bebfd9a23e74fa.js"></script>

And what's really nice is, since the `TypeId` of a given type is a constant, that means after rustc has monomorphized the function, downcasting becomes comparing a constant (`TypeId` of the generic) to another constant (the specified `TypeId`, like `File` in the above code) and since compilers are smart, they see that for each function only one possibility of the branch can ever be hit, so the other half of the branch can be removed entirely. This optimization/class of optimizations is called "dead code elimination", and, when used in a blanket trait implementation, gives us a limited form of specialization.

### Testing to Ensure it Works

Ok so the compiler theoretically making our code as good as if we hand-wrote code for each of our types is all fine and good, but relying on LLVM to do the right thing always is... not necessarily the best way to ensure our software is fast.

Like any hallmark of optimization, let's look at the 3 most common means of testing:

* Benchmarking
* Reading assembly
* Terrifying hacks from hell

And as the test subject I'll be applying it to my motivating use-case, speeding up my parsing library [binread](https://docs.rs/binread). One problem with it is that since it's very trait-heavy, occasionally the lack of specialization can have a serious performance penalty. For example before, `Vec<u8>` would simply parse `u8`s one at a time. One could imagine why this is a bad idea, especially in the specific case of `Vec<u8>`, where no actual parsing needs to be done, so there's no point doing one item at a time!

First up, **benchmarking**! The classic, the bread and butter, etc etc. For the test, we'll read ~48 MiB from `/dev/zero` unbuffered into a Vec.

Before:

```
  Time (mean ± σ):      9.815 s ±  0.076 s    [User: 3.005 s, System: 6.798 s]
  Range (min … max):    9.710 s …  9.953 s    10 runs
```

After:

```
  Time (mean ± σ):      21.4 ms ±   2.4 ms    [User: 0.6 ms, System: 20.8 ms]
  Range (min … max):    15.5 ms …  26.8 ms    122 runs
```

While this certainly is a performance boost, it doesn't really tell us much about if our code is actually optimizing out the branch, just that our specialization emulation is actually yielding a massive speedup (Albeit in an example I would call contrived if I didn't experience it firsthand).

Ok so... I guess time to read assembly! Is what I'd say if I actually wanted to talk about that in this blog post, which I don't. I personally read through the disassembly for the above benchmark to make sure everything was going as expected and my recommendation is: for this specific case, don't :)

In all seriousness though, `cargo-asm`, Rust playground's asm output, godbolt, etc. are all great tools for looking into it, however the above code is very I/O focused and involves errors, so it's hard to get minimal output to comb through.

And after all, why do something by hand when the compiler can do it for you?

<script src="https://gist.github.com/jam1garner/ebd32ffb3aa78cbcfed1da1aff7eb95f.js"></script>

To start off: if you already see where this is going, you may be entitled to financial compensation.

Now, what's going on here? Well, our goal is to ensure the incorrect branch is never compiled into the final executable. A fun way to check that is to cause a linker error whenever the bad branch is compiled into the binary by just... trying to call a non-existant function. So if we move it to the opposite branch (i.e. the only one of the two we expect to get compiled in) we get a fun error like this:

```
  = note: /usr/bin/ld: /home/jam/dev/binrw_bench/target/release/deps/binrw_bench-d4d40848d486b3c3.binrw_bench.3e7o84sm-cgu.2.rcgu.o: in function `binrw::binread_impls::<impl binrw::BinRead for alloc::vec::Vec<B>>::read_options':
          binrw_bench.3e7o84sm-cgu.2:(.text._ZN5binrw13binread_impls69_$LT$impl$u20$binrw..BinRead$u20$for$u20$alloc..vec..Vec$LT$B$GT$$GT$12read_options17hca5f5dc1876891fcE+0x7b): undefined reference to `should_not_be_included'
          collect2: error: ld returned 1 exit status
```

...proceeded by quite literally a full screen of garbage nobody cares about, showing that this invalid function call could not be optimized out and, if this was the branch we wanted to avoid, we would need to rethink our approach.

Some closing thoughts in no particular order:

* Like any hack, this is of course only applicable for a small subset of what specialization enables
* Ultimately the branch won't matter much performance-wise, but it's great to optimize it out for the purposes of reducing the binary bloat issues that can come with monomorphization.
* This part of the blog post is honestly longer than I expected but, contrary to what the order would indicate, I have already written most of the rest of the blog post so...

## <a name="installers"></a> Building Rust Windows Installers in CI

Rust tooling tends to be nice in surprising ways, and Windows installers is no exception. [`cargo-wix`](https://github.com/volks73/cargo-wix) is a cargo subcommand for building Windows installers using the WiX toolset.

The install is rather simple:

```
cargo install cargo-wix
```

The setup for a given project is rather simple:

```
cargo wix init
```

And actually building the installer is rather simple:

```
cargo wix
```

One thing that I *didn't* find to be simple was setting up a workflow for Github Actions to build and release an installer any time I made a new release. WiX has to be installed using a GUI, an unfortunate matter common in the Windows sphere sadly. (Yes, I do see the irony in complaining about this while instructing you how to build a graphical installer)

Fortunately though, they have a seemingly undocumented ability for headless install:

```
curl -OLS https://github.com/wixtoolset/wix3/releases/download/wix3111rtm/wix311.exe
.\wix311.exe /install /quiet /norestart
```

If you'd like to see my full workflow for this (including uploading as a release), the workflow file [can be found here](https://github.com/jam1garner/cargo-mextk/blob/master/.github/workflows/release.yml).

## <a name="macro-coverage"></a> Measurement of Macro Test Coverage

This one was most definitely not enough to be an actual blogpost, and is really just some assorted tips that might help save others a bit of time. But here's basically how I went about setting up test coverage. Note that basically all of these steps assume that your [macro-based crate](https://doc.rust-lang.org/reference/procedural-macros.html) is split between two crates: one being an actual library, and the other being a proc-macro itself. For those who aren't familiar, this is typically needed due to current Rust limitations as any traits/types the proc macro will need need to be in a non-macro crate, while any macros have to live in their own crate.

Some prerequisites:

1. Use a cargo workspace. Not only will this help manage everything, it allows the two crates to know relative paths for each other.
2. Integration tests will need to be under the non-macro crate, to allow them to actually have access to the traits/types required as well as the macro.

The following instructions use [tarpaulin](https://github.com/xd009642/tarpaulin), however they should more or less generalize to any tooling.

So first up, the problem to be solved: proc macros don't get measured by our integration tests. This makes sense as they are both only a dependency from the perspective of our binary *and* they aren't even compiled into the final binary, they are just a part of the rustc process.

So we need a way to run our macro as a part of our tests themselves, fortunately the library [runtime_macros_derive](https://docs.rs/runtime-macros-derive) has us covered, it does exactly that!

Some things of note:

* It runs under the context of the proc-macro tests, which is why we want to be able to construct relative paths to our non-macro library
* It doesn't actually run the tests associated, it just runs the tokens through the derive macro, which is why the tests themselves need to be in the other library

The core of this "test" will look something like this:

<script src="https://gist.github.com/jam1garner/32ba8cd042bd7888082fff7115fb7ebd.js"></script>

where `derive_macro_internal` is a function which takes a `syn::DeriveInput` (rather than a `proc_macro::TokenStream`). You'll typically want your macro itself to look something like:

<script src="https://gist.github.com/jam1garner/0f2fe40c496c2eabdf48f8cdd1efbbb8.js"></script>

Lastly, we need to run tarpaulin for the entire workspace:

```
cargo tarpaulin --workspace
```

(this should be done from the root of the workspace)

Now your integration tests should show up in your coverage reports, even on the proc-macro end!

And, fortunately, cargo-tarpaulin is [really easy to integrate into a Github Action workflow](https://github.com/jam1garner/binread/blob/7f4d0dd0d04c55d036707481b78cf9ae0eebef8d/.github/workflows/coverage.yml), making it easy to keep your open source project's test coverage up to date.
