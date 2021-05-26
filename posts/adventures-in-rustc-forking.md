<!--timestamp:1622050343-->

# Adventures in rustc Forking

![](/img/ci_failed.png)

Quick introduction for anyone who isn't familiar: [Skyline] is a framework for developing mods for Nintendo Switch games, allowing users to add/patch code to their favorite games. [skyline-rs] was a library I created to allow for writing mods in ergonomic and idiomatic Rust. [cargo skyline] is a cargo subcommand for aiding in building Skyline plugins using skyline-rs. And lastly [rust-std-skyline] is a fork of the Rust standard library designed for use on the Nintendo Switch. Ok, enough introductions!

[Skyline]: https://github.com/skyline-dev/skyline
[skyline-rs]: https://github.com/ultimate-research/skyline-rs
[cargo skyline]: https://github.com/jam1garner/cargo-skyline
[rust-std-skyline]: https://github.com/jam1garner/rust-std-skyline

A while back I made a blog post [on using Rust for Modding Smash Ultimate]. Since then, there are even more people using skyline-rs for mods for a variety of games. Even Skyline itself has started using Rust internally. One part of my previous post about using Rust for game modding that didn't quite make the final cut was a section discussing [xargo], a (deprecated) utility for cross compiling Rust with a modify copy of the Rust standard library.

[on using Rust for Modding Smash Ultimate]: https://jam1.re/blog/rust-for-game-modding
[xargo]: https://github.com/japaric/xargo

The thing about xargo is, it's *kinda* deprecated for a reason. I would argue its primary use-case is custom targets which use an existing copy of the Rust sysroot and building core, compiler_builtins, and alloc for a custom target. For this use case it was superseded by [cargo-xbuild], which is a fork of xargo that is popular for Rust OSdev fans, but even *that* is not the hip new option. It was *itself* superseded by a new built-in Cargo feature known as "std-aware cargo" (or [-Zbuild-std]). So you might (extremely reasonably) be asking: why, jam, are you using this fifth-cousin twice-deprecated equivalent of a cargo feature? Well, the problem with std-aware cargo in its current state is that *it's not really ready for building custom std's*. This is a slight problem when our goal is building a custom std fork.

[cargo-xbuild]: https://github.com/rust-osdev/cargo-xbuild
[-Zbuild-std]: https://github.com/rust-lang/wg-cargo-std-aware

## Using Xargo

So when I first implemented rust-std-skyline, xargo was slightly less twice-deprecated, so I stuck with it. After all, my only other option was distributing custom Rust toolchai—oh no, no no no no no no. I don't like the direction this blog post is heading.

Ok so. Look. xargo is wonderful and I'm very grateful to its maintainers, there's like a 50% chance I would've given up a long while ago without xargo. But I'm going to be honest, using xargo has caused me nothing but pain. The initial setup of getting it working for my target required so much endless fiddling and tweaking that for a while I ended up maintaining a fork of it until I could find workarounds for all the little bits of paths not being quite right, looking for files in whatever the current working directory was during some build step that modified the CWD constantly. And even after that I needed a decently heavy wrapper around xargo (cargo-skyline) in order to even be able to build due to all the tweaking that had to happen.

And that's just the surface. All this tweaking made it incredibly hard to stay up to date with [rust-lang/rust] master. And a large rust-lang/rust restructure months ago made it near impossbile. My solution? Force users to pin an extremely specific day's nightly build for all eternity!

[rust-lang/rust]: https://github.com/rust-lang/rust

But this creates a fundamental issue: over time, more and more crates will depend on features that weren't stabilized, or worse, even implemented! This meant an increasing amount of builds breaking, hacky version-pinning fixes, `cargo update` breakage, constantly diverging! More and more, I and others wanted to get back in sync with upstream. So it was time to ditch xargo!

## Forward Porting

This part was really boring let's just skip it. Huge thanks to [Devin] and [Raytwo] for helping with this part. It was a massive deal of fixing merge conflicts, doing this largest double-rebase of my life, and having a 30 minute long tweak-save-compile loop. I have literally no interesting points to make here.

[Devin]: https://github.com/inspier
[Raytwo]: https://github.com/raytwo

## Fixing Build Issues

I'm going to have to ask a lot of you here audience. I'm going to take semi-reasonable problems and apply horrifying disaster fixes to them. I just need you to cover your eyes for those parts and forgive me for what I'm about to do.

One of the initial issues I hit was that the linkage step for the cross-compiled `std` did *not* play well with the concoction of linker scripts, target-specific ld-only linker flags, and cross compiling. It attempted to use `cc` (the system C compiler) for linking, but my linux system linker wasn't going to cut it. So I set my linker in [config.toml] (the rustc compilation config) to `aarch64-none-elf-ld`, a fork of `ld` provided by [devkitPro], but that just resulted in it not liking `gcc` flags being passed directly to the linker, so I set it to `aarch64-none-elf-gcc`, but uh nope that made things worse.

[config.toml]: https://github.com/rust-lang/rust/blob/f6a28aa4036415d8aa713bf707842779b709935e/config.toml.example
[devkitPro]: https://devkitpro.org/

I didn't want a dependence on devkitpro anyways, so the easy route not working out is a bit of a blessing. But looking into things further, I found that it wasn't (as far as I could tell) possible to get x.py (the Rust build script) to output a combination of linker flags compatible with any of the linkers I had on hand. So I had to get a bit creative. I made my "own" linker.


If this sounds like a bad idea, it was! And boy did scope-creep kick in. `fake-ld` is now a disgusting combination of a few things:

* A `cc` -> `gcc` forwarder
* A `cc` -> `zig cc` forwarder
* A "linker" that modifies arguments in a variety of ways then using `ld.lld` internally to handle actually linking
* A cross-platform zip file utility

After finishing initial implementations of these, I had things building/working locally!

Problem is... I don't have a Windows or Mac to make/package builds of rustc, and I have a lot of Windows/Mac consumers. But building/packaging by hand every time would be painful anyways! So either way you cut it I needed Continuous Integration to build. And I'm not sure if you've noticed but this is a rather precarious build setup. But fear not, it gets worse! Due to [various bugs](https://github.com/rust-lang/rust/issues/85593) with cross compiling rustc ([one of which I actually submitted a PR for!](https://github.com/rust-lang/rust/pull/85590)) it doesn't appear possible to produce these builds without actually building them on the target themselves.

Which also meant I had to roll my own CI pipeline for packaging/distributing builds of rustc. And the pipeline had to work cross platform. And that is where the pain lies!

## More CI, More Problems

![](/img/ci_failed_alot.png)

This was a multi-week debugging experience. Caching was constantly failing. Each build environment had its own requirements. And Windows' requirements seemed to span for pages and pages. Hell, as I write this, a new ICE (Internal Compiler Error) for MacOS just dropped on master.

![](/img/macos_ice.png)

Ultimately, after days upon days of tweaking CI every 3 hours, I finally got all platforms building. Aaaaaaand then I get hit with this beauty:

![](/img/ci_horror.png)

And so after a bunch of fiddling with deleting as much as possible after building on Windows, I finally produced a single successful release.

## Retooling

Now for the fun part—reworking cargo-skyline for the new toolchain. This time around things were a lot easier to rework. Just needed to rewrite my update code to use the github API to download the latest build of my toolchain and rewrite the build portion of the code to move from xargo to force-using the newly linked rustup toolchain.

And with that, I'd like to announce the release of cargo-skyline 2.0.0, to install just update using cargo:

```
cargo install cargo-skyline --force
```

## Migration

A lot of care has been put into ensuring breakage is minimal when upgrading to cargo-skyline 2.0, and for a lot of projects they should *mostly* "just work" when built with the new version. 

If you have any projects fail to build, or simply want to remove old files that are no longer needed in cargo-skyline 2.0 projects due to the removal of the Xargo dependency, try the following in the root of your project:

```
cargo skyline clean-project
```

This will:

* Remove Xargo.toml and aarch64-skyline-switch.json (Files only needed for Xargo)
* Remove `.cargo/config` if and only if it matches the default cargo-skyline template
* Remove Cargo.lock to prevent stale dependencies
* Run `cargo clean` to delete outdated dependencies and save you space

## More

Thanks for reading, if you'd like to see more like this, [follow me on twitter].

[follow me on twitter]: https://twitter.com/jam1garner
