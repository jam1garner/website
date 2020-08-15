<!--timestamp:1597510638-->

# Why Rust's Unsafe Works

![](https://rustacean.net/more-crabby-things/corro.svg)

Like many great things in life, this post has been inspired by spite. Reddit, more specifically r/programming, has gotten on my last nerve. So my goal is simple: I want to outline why Rust's `unsafe` keyword works, while similar measures in C/C++ don't.

## The Headache of C

I, like many others, have the misfortune of a day job involving C. It's not a bad language, I enjoy it in the same way I like writing assembly or esoteric languages: as a fun, challenging puzzle. I can't say I've ever enjoyed maintaining C, nor do I really think anyone has been able to in a long time. And, coming from my background as a security professional... C has a lot of problems when it comes to security. Everyone knows, it's not secret, I'm not here to lecture you about how if you don't rewrite everything in Rust everything will explode.

Let's take a look at an extremely simple C program:

<script src="https://gist.github.com/jam1garner/ef5f7a6d752ef9a09e94b7a84d6a70ff.js"></script>

Simple enough. A function returns a string. Now down the road, this function gets modified by a coworker to return a name generated at runtime. By the magic of programming, I don't need to change any calls to `get_name`! The function signature didn't change! A string is a string, all good:

<script src="https://gist.github.com/jam1garner/3c7b2a494d6008adaeaefba6c0ce1e71.js"></script>

The programmers who have spent enough time in the coal mine may immediately notice a problem with my program. There's not a single free! What a fool I am, leaking memory all over the floor. Silly me. But wait, I thought it wasn't going to break things at the call site? :(

This is the problem C's memory management: it's a silent contract you must fullfill. If you're lucky, your coworker left a comment above the function to tell you ownership of the string is passed to the calling function and is responsible for freeing it. If you're unlucky, you either had to go read the function definition or just say "whatever, a memory leak isn't as bad as a use-after-free".

"Wait a second, did you just say "ownership"? I thought that was a Rust thing!"

Ah but it isn't! Ownership is a pretty universal concept, but in other languages you have to keep track of it yourself. (and worse yet wait till you find about how it doesn't just apply to memory-management...)

Tangent aside, back to fixing the program:

<script src="https://gist.github.com/jam1garner/a01945af23461db0912d1f54130b8442.js"></script>

I added some additional functionality (capitalization!) but just ignore it. We have our `free` now, I tested it, and everything is working now! :)

<script src="https://gist.github.com/jam1garner/e25c345343066441f085898aea2236de.js"></script>

...uh oh.

Did my coworker... just revert his commit?

```
Segmentation fault (core dumped)
```

ughhhhhhhh. But I just finished adding all the `free`s!

Ok I'll stop beating a dead horse. Point is: silent contracts suck, and they aren't scalable. In C it's not possible to denote ownership, you don't even have [RAII](https://en.wikipedia.org/wiki/Resource_acquisition_is_initialization) to carry you. And for those who aren't especially careful viewers: before, I implicitly cast a `const char*` to a `char*`, even the concept of "oh hey you can't edit this data" is poorly encoded in C. And sure that'll give you a warning... if whoever wrote the function properly marked the return as `const`... anyone who has dealt with a C codebase larger than they can memorize the API of has run into these silent contracts.

And RAII/Smart pointers/etc. certainly help C++ a lot in this regard, but ultimately those fail when you need to share memory safely and efficiently. It has guard rails, but like C's failure to uphold `const`-ness, any non-trivial usecase will eventually plough right through them. And, just like Rust, they also have... adoption issues. Your decades-old C++ application isn't going to be using smart pointers. Converting it is a non-trivial task.

## The (Good) Headache of Rust

Now, to avoid this coming off as language bashing, I probably need to actually talk about Rust! And, more importantly, debunking the counterpoint I hate hearing the most:

> Sure, Rust is "safe", but `unsafe` exists, and most of the standard library is `unsafe`!

Ok so we're gonna need to talk about two things: encapsulation and locality.

## Encapsulation

I'm sure if you're at all familiar with Object-Oriented Programming (or have sat in a freshmen CS lecture for more than 5 minutes) you have heard of encapsulation. If you haven't, you're not missing out on anything. Encapsulation, or hiding data inside a larger grouping and only providing access via methods, is basically where OOP principles start and end in Rust.

It's a rather useful tool for organizing your code and building up solid abstractions. If you reveal too much, you create a leaky abstraction, which can prevent you from changing the implementation down the road. In our C example, since our memory management isn't abstracted away, we can't change certain implementation details, such as where our data is stored. By not exposing too much of the implementation, we can change how things work for performance or a changing use-case.

Now how does this have anything to do with `unsafe`?

The ultimate goal of Rust is not to remove the dangerous pointy bits entirely as, at some level, we need to be able to just... access memory and other resources. The goal of Rust is actually to abstract all the `unsafe` away. When thinking about security, you need to think about "attack surface", or what parts of the program we can interact with. Things like parsers are a great attack surface because:

1. They are often accessible by an attacker
2. Parsing often requires complex logic directly affected by attacker-provided data

You can actually go on to break this down further by breaking up the traditional attack surface into the "attack surface" (the part of the program you can directly influence code in) and the "unsafe layer", which is the part of the code that is relied on by the attack surface but isn't accessible *and* has the potential for bugs. In C, these would be the same thing: arrays in C aren't abstracted at all, so if you read a variable number of items, you need to make sure all invariants are upheld because you are operating in the unsafe layer, where bugs can happen.

Now lets compare this to Rust: a `Vec` is made up of `unsafe` code and therefore has the potential for bugs, making it the unsafe layer. However a `Vec` encapsulates its data, I have to use methods to access it. If there are no bugs in Rust's libstd `Vec` implementation, then nothing I do to a `Vec` will cause memory corruption. So then it just becomes a matter of ensuring `Vec` is implemented correctly (a relatively small unsafe layer) in order to ensure arbitrarily large `Vec`-based attack surfaces are sound. The result is simple: putting the unsafe layer as far as possible from the attack surface. More on this in a bit.

Now this line of reasoning usually introduces a bit of a logical fallacy I've seen raised a few times and that is "if all of safe Rust is built on unsafe Rust, doesn't that make all of the Rust unsafe?". I'm gonna take that question and make it way more complicated than it needs to be but here goes:

There is a finite set of inputs a function can take, and a (possibly empty) subset of those will cause memory safety vulnerabilities, however if it is logically impossible to pass any of the unsound inputs, then it is logically impossible to cause memory corruption. For this example, lets substitute memory safety with something easier to reason about: panics.

<script src="https://gist.github.com/jam1garner/f6b39d9cf4be826b167db2e04ecd9cc3.js"></script>

In this example, there is a single input in the set that will cause a panic (0). If our goal is to make it impossible to panic, then, assuming the input can be anything, we have failed. However if we create a second "safe wrapper" around it, to prevent invalid inputs, it will never panic:

<script src="https://gist.github.com/jam1garner/9a309e8a0bf17b8c9c40e4187fc7a13d.js"></script>

substituting back unsafety for panicking, we have removed all unsoundness from our program by encapsulating the unsafety. No matter what you do with the safe wrapper, you won't cause the panic. While there is a larger number of invariants than "not zero" that need to be upheld for memory safety, as well as a larger number of inputs, I think you'll find encapsulation still greatly combats unsafety, especially because it reduces the size of the afformentioned "unsafe layer" by an order of magnitude, allowing for a greater certainty of safety at a reduced labor cost of inspection.

## Locality

One of the greatest features of Rust is its increased design focus on locality, which is to say, it's easier to reason about a function without looking outside the function. An example from our C program earlier is that our memory management doesn't have locality: whether or not I need to `free` the memory after use in one function is dependent on the implementation of another function. That is to say that looking at the function signature for a C program is not enough to reason about how memory passed to and from it should be treated.

The alternative provided by Rust and, to a lesser extent, C++ smart pointers, is encoding the memory behavior in the return type. In Rust there is the `Box<T>` type, which indicates the value is heap allocated and owned by whoever owns the `Box`. If I pass you a Box, I no longer own it and therefore you are resposible for free-ing it. So if a function returns a `Box`, it is explicitly saying to everyone that calls it that they are responsible for freeing the memory returned. Similarly, the `String` type indicates heap allocation and ownership, while `&'static str` represents an immutable string baked into the binary.

If we rewrite my original function in C we get:

<script src="https://gist.github.com/jam1garner/d199057d232d42cc1f0451834a58fa2f.js"></script>

and so if we change the function signature to be a heap-allocated string, that will be a breaking change as encoded by the type system. So if my coworker changes the code from a heap-allocated `String` back to a hardcoded `&'static str'` literal, then if I wrote code which modifies the string, he'll get a compiler error and know to update my code.

This, however, also results in not just more memory-safe code, but also easier to refactor as I can confidentally change it from an `&'static str` to a `String` and if it would cause breaking changes, the compiler will immediately tell me, not a user's bug report in 2 months when I've already forgotten what changes I made.

This is taken even further when looking at borrowing more specifically, as lifetime specification allows you to encode usage rules to further ensure global memory safety by just proving local memory safety and lifetime assurances. This ability to have everything provable locally results in significantly less work to show that everything is safe. First off, as previously mentioned, it allows you to prove the entirety of your program safe, so long as your safe abstractions over unsafe code are which meaning orders of magnitude less code to audit. Second off, since these abstraction's safety isn't dependent on how they are used, the amount of auditing required grows linearly with the size of the codebase, rather than exponentially, as you don't have to inspect the interactions between abstractions and uses.

## How Encapsulation and Locality Work Together

Rust allows separating out all the code that needs careful scrutiny (and thanks to the `unsafe` keyword, it's even grep-able for when things occasionally go wrong!). And one really nice bonus to all of this is that, since the standard library is very thoroughly tested you will only incur issues if you *really* go out of your way to. Let's go through some of the recent high-profile unsoundness bugs from the Rust libstd:

1. [Unsoundness in Pin](https://internals.rust-lang.org/t/unsoundness-in-pin/11311) - As described by [the wonderful primary designer of Pin](https://news.ycombinator.com/item?id=21665809), "the code [...] is all obviously pathological & contrived", stating it could not have a "practical impact on users". 
2. [PartialEq for RangeInclusive is unsound](https://github.com/rust-lang/rust/issues/67194) - This bug, while significantly less so, is still rather contrived. To the best of my ability, I can't think of reasonable code that would trigger it. And even in that situation, the code would just malfunction consistently, resulting in it being caught before the code is actually shipped.


While these are certainly massive issues if your threat model is "run arbitrary user-provided safe Rust on an unsandboxed system"... that just isn't a threat model that exists. Let's take a look at one more example.

3. [Segfault in VecDeque](https://github.com/rust-lang/rust/issues/44800) - Finally, an issue with some substance. Let's look at the minimal reproduction:

<script src="https://gist.github.com/jam1garner/9a16310df42415be9dd5ed136893554d.js"></script>

Yikes! That definitely could be triggered in normal code! And it was present for 2 years??

Ok now that I've provided my legally-required "Rust is flawed" part of the blog post, here's my thoughts on the matter. In exploitation your goal is often fitting through a really tight window of opportunity. You have to meet the right conditions in the right order all while fitting in the constraints given, all while typing using a sideways Wiimote. That is to say, you often have rather limited control over what you can do. While it is definitely possible to exploit the above vulnerability in some reasonable program, it's also rather unlikely in any given use of `VecDeque` that this is exploitable. Why? Because this requires a specific sequence of events to occur, and in most cases you just don't have the tools needed to cause `with_capacity` -> `push_{front, back}` -> `reserve` -> `push_{front, back}`. This is why having as large as possible a separation between attack surface and unsafe layer is useful, you reduce the amount of control over the unsafe code an attacker has, reducing the probability a vulnerability is exploitable while also making it harder to exploit when it is.

So that's my thoughts on why even an `unsafe` foundation can build up to rather safe and hard to exploit code. If you'd like to tell me how poorly thought out this is or explain to me how Rust is actually bad and unsafe, let me know on [twitter](https://twitter.com/jam1garner)!
