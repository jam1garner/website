<!--timestamp:1597510639-->

# Rust for Modding Smash Ultimate

![](/img/smash_x_rust.png)

I'm gonna be upfront with you here, I'm one of those people obsessed with Rust. This is gonna be a post ripe with bias. Very sorry if you're not a fan, but for me it reinvigorated my love of programming I lost years ago, I hope viewing my evangelism under that lense helps.

So recently I've been working on a lot of projects. All of them are in Rust. Some background knowledge for anyone not familiar with my work: in my freetime I write a lot of tooling for modding games, currently targetting the switch, primarily "Super Smash Brothers Ultimate". Today I'm planning on talking about my experience using Rust for game modding (spoiler: it was extremely positive) and announcing some new project releases.

## Old News

So back in April, having not been more than 100 meters from my bed in a month due to the looming pandemic, I needed a new project to help cope. (Is "doing work for free" a good coping mechanism? Don't ask me, I'm not licensed for therapy where you live!)

Enter [Skyline](https://github.com/skyline-dev/skyline): An environment for runtime hooking/code patching on the switch. I was immediately intrigued having worked on other code patching projects in the past (most of which targetting the oft-unloved Wii U). I was especially intrigued when I learned that the main usecase this was targetting was my "home game" of Ultimate.

However. There was a problem.

![Main language C++](https://cdn.discordapp.com/attachments/376971848555954187/765606858371104808/unknown.png)


My old enemy. C++. I'll be straight with you—I'm not a fan. It's a language that has done great things, but honestly I find working with it excrutiating after years of using it. It's full of footguns, bad dependency management, and decades of stacking new features into a tower of technical debt. This is obviously just a product of the environment and the time it's been produced over, however that doesn't mean I should give it a free pass at my love.

Skyline wasn't quite in a usable state at this point, and I decided to use that to my advantage. I discussed wanting to use Rust with Shadow, the developer, and they weren't opposed to the idea since it would fall pretty squarely inline with supporting C. So I got to work on compiling Rust targetting the Switch.

Thanks to the help of a few people:

* [roblabla](https://github.com/roblabla)
* [leo60228](https://github.com/leo60228)
* [Thog](https://github.com/Thog)
* [kitlith](https://github.com/kitlith)
* [FenrirWolf](https://github.com/FenrirWolf)

I was able to get Rust compiling targetting the switch rather quickly. First I started with `no_std` targetting aarch64, then moving on to using [xargo](https://github.com/japaric/xargo) for creating a port of std to the switch. No part of the port was particularly interesting, however I cannot begin to explain how much I was aided by the Rust Project's focus on cross compilation. Porting libstd was surprisingly painless and cargo has been as amazing as ever. I have never had such pleasure porting something to a new platform, and that is in no small part due to Rust's libstd having such intentional design abstracting over everything I'd want between platforms and nothing more.

To add some detail to the massive undertaking I just glossed over: this libstd port was designed around running Rust under the context of consumer games. It links against parts of the Nintendo SDK that I and others (namely Shadow, the developers of [Starlight](https://github.com/shadowninja108/Starlight), and [Raytwo](https://github.com/Raytwo), among others) have reverse engineered. This means that in theory this would also allow for Rust to be used for writing consumer games, not that I imagine Nintendo would be a fan of those submitting such a game for review.

## Old(ish) News

Once I had the ability to write Rust for the Switch, it practically became an addiction. The reverse engineering -> implementing -> using loop gave such a consistent dopamine hit that it has essentially become my sole hobby.

Another thing Rust has made rather pleasant has been the dependency management. Being able to use existing libraries as well as compartmentalizing our own code into libraries has been an absolute blessing in growing the tooling for our mods in a publically-usable way.

For example, making bindings to [smash itself](https://ultimate-research.github.io/skyline-rs-template/doc/smash/index.html) and not having to teach people making mods (who tend to be less familiar with traditional programming tools) how to use git submodules has been wonderful. Since cargo handles downloading and using dependencies, all the source for mods is significantly better organized on average and has made open-sourcing mods something anyone can do. And thanks to rustdoc, for once finding things when writing mods using existing code feels... actually doable.

The real power of Rust, however, has come from a lot of its language constructs. For example, in Smash movesets are controlled by "ACMD", or "Animation Command", a domain-specific language created by the developers in order to allow for ease of development. It is a coroutine-based language focused on mapping frame timings to actions. In Smash Ultimate, they changed up how ACMD works a bit by changing it from its own interpreted scripting language to being built on top of lua, then run through lua2cpp and compiled as a dynamic library.

The creates a lot of usability problems when writing our own movesets. Interacting with a smash API on top of the lua C++ API is... not fun. It's verbose, messy, and requires an understanding of Lua, Smash, and ACMD. Early solutions to this involved making a C++ wrapper around them, however that caused a lot of issues in trying to emulate the yielding of corountines given our limited control over the datastructures we had access to for state.

However thanks to Rust, we can do better! While any Rustaceans following me might think this is the perfect place for async/await, I'd agree with you! But unfortunately I came to the conclusion that would be overkill, and only complicate things for the target audience (new programmers writing moveset edits). So the solution I ended up coming up with is [skyline-acmd](https://github.com/ultimate-research/skyline-acmd), a macro-based solution to writing ACMD. Instead of writing Rust, you write ACMD in the syntax provided by [Rubendal's data viewer](https://rubendal.github.io/ssbu/#/Character/Falco), which provides a decompiled view of Smash movesets.

<script src="https://gist.github.com/jam1garner/488ef770e0b0f36847118f470af2e69e.js"></script>

This allows for writing movesets in a way that removes the complexity of Rust and the Lua interface and simply let you work with something about equivelant to what the developers themselves had to work with. [I even gave a talk at RustConf on macros where I briefly talk about it!](https://www.youtube.com/watch?v=dZiWkbnaQe8)

## New News

Now onto some of the projects I've been working on. All of these have been released alongside the blogpost.

### TCP Sockets

This is a super minor addition but with some help from Raytwo we've gotten Rust's std::net supported, meaning plugins can now communicate with the outside world, interact remotely with PC applications and more! Reversing the Nintendo SDK enough to have this work was a pain, but having it come together has opened a lot of doors.

### skyline-web

One feature of the Switch SDK is nn::web, a web browser you can use in-game. It features the ability to overlay over the game, run javascript, render anything a relatively new webkit supports, and is customized for use with controllers/touchscreens. [`skyline-web`](https://github.com/skyline-rs/skyline-web) is a set of high-level Rust bindings for allowing developers making Skyline plugins to use the web browser. This is great for adding custom menus to games using the well-documented and easy to use/test interface of the web.

<script src="https://gist.github.com/jam1garner/b9c4cb87440a9bc594c51dd392593c00.js"></script>

(made with help from Raytwo)

### switchtml

skyline-web is perfect for popups (as it stops the game but also can overlay over it), however fitting in with the Switch UI itself is preferrable. To help people accomplish this, I've published a [set of examples](https://github.com/jam1garner/switchtml) for making Switch-styled UI.

### skyline-update

One problem we have encountered in developing mods is that people use outdated versions because it's a lot of effort to download the files and send them over to your switch. This leads to bug reports from outdated versions being rather frequent. In order to solve this problem I've created [skyline-update](https://github.com/skyline-rs/skyline-update/), an update server, update protocol, and update library designed for updating mods from the Switch itself.

Anyone can run their own update server and serve as many plugins as they'd like, and users can update your mod in the click of a button:

![Screenshot of the update prompt](https://media.discordapp.net/attachments/205341690158907393/765656044277334026/2020101321231200-0E7DF678130F4F0FA2C88AE72B47AFDF.jpg)

This only requires a small code addition to add it to an existing plugin:

<script src="https://gist.github.com/jam1garner/ce21ab8268e58136f8826406c68d782d.js"></script>

That's all it takes, and you can even add an opt-in beta channel to your plugin.

### arcropolis 0.9.0

Arcropolis, for those who aren't familiar, is a mod for Smash Ultimate written in Rust that allows for modifying files in Smash Ultimate's 16 GB archive file without filesize/compression limitations.

It's now getting a [0.9.0 release](https://github.com/Raytwo/ARCropolis/releases) (just in time for Smash 9.0.0).

A quick changelog:

* Added an autoupdater using skyline-update!
* It fixes a bunch of crashes
* Adds the initial version of the callbacks API
* Various code improvements
* Displays that you're running arcropolis on the title screen
* Adds colored logs using [owo-colors](https://docs.rs/owo-colors) (a terminal colors library by yours truly)
* Adds support for new filetypes (motion_list, numshexb)
* Improves load times
* Uses [rayon](https://docs.rs/rayon) for multithreading some of the load times (temporarily removed)

### Helios

One problem with skyline-update is that it's only really useful for skyline plugins! If your mod doesn't have an associated skyline plugin, you can't really have an updater. [Helios](https://github.com/skyline-rs/helios) is the solution to that. It's a configuration-based general-purpose updater that allows you to setup updates for mods using just a simple config:

```
name = "helios_test"
version = "1.0.0"
server_ip = "999.999.999.999"
```

All helios mods will check for an update at the same time then be displayed in a single popup. Mod creators can then distribute helios configs alongside their mod so helios users will be able to update it without reinstalling the mod themselves.

### img2nutexb

Smash texture format work is tedious. You have to save as DDS with specific settings then use Windows-only tools for converting that DDS file to nutexb. I made a cross-platform alternative: [img2nutexb](https://github.com/jam1garner/img2nutexb), a converter for converting multiple formats of images/textures into nutexb. Along with this comes a [nutexb library](https://github.com/jam1garner/nutexb) for any Rust projects to use to work with the format.


### Skin Converter Website

Want to use img2nutexb for converting Minecraft skins over to smash but hate the command line? Thanks to a collaboration with [CoolSonicKirby](https://github.com/coolsonickirby) you don't need anything more than a minecraft username to convert your own skin. Check it out [here](https://smashultimatetools.com/skinConverter)!

### skyline-rs Organization

There's now a [github organization](https://github.com/skyline-rs) for storing all the forks of projects made to work on the Switch. For example if you want to use [rand](https://docs.rs/rand/), you'll find it has a dependency on [getrandom](https://docs.rs/getrandom) (which doesn't support the switch). So in order to use it, you simply patch it:

```
[patch.crates-io]
getrandom = { git = "https://github.com/skyline-rs/getrandom" }
```

cargo's patch system has been an absolute delight in porting libraries over. Since the fancy high level crates will be built on top of small primitive crates focused on being cross-platform, sometimes porting high-quality crates simply means forking the primitve crate and porting a few small functions to your platform. 


In order to help facilitate some of the great libraries for use on the switch, we've made an [awesome-libraries](https://github.com/skyline-rs/awesome-libraries) repository to help keep track of what works on the switch as well as any additional patches it requires to work.

We even have https requests working with [minreq](https://docs.rs/minreq) by simply patching [ring](https://docs.rs/ring/0.16.15/ring/)!

### Training Modpack Menu

![A shield toggle menu](https://cdn.discordapp.com/attachments/349043564237553664/766034054197542952/2020101413242900-0E7DF678130F4F0FA2C88AE72B47AFDF.jpg)

This menu, powered by skyline-web, is coming to the [Smash Ultimate Training Modpack](https://github.com/jugeeya/UltimateTrainingModpack) thanks to a collaboration with [jugeeya](https://github.com/jugeeya). It nearly removes the need for a heavy dependency on a C++ sysmodule.

### Minecraft Skin Downloader

So Steve from Minecraft just got added to Smash and yet, unlike Minecraft itself, you can't use your own skin! This mod intends to fix that by providing a user interface for downloading skins from Minecraft: Java Edition from your minecraft username and then converting them to Smash skins. Using a custom menu for picking your skin and the nutexb library for converting textures on-console, you can use your own personal minecraft skin without knowing anything about modding.

<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/setahMr5mVw" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

**How does it work?**

The menu is skyline-web powered, the download uses minreq to communicate with the minecraft API, nutexb is used to convert the texture, and the new arcropolis plugin-to-plugin API is used for replacing the file!

Being able to use the same library both for a CLI and for an on-console mod has been absolutely mind blowing and easy.

[Source code](https://github.com/jam1garner/smash-minecraft-skins)

[Download link](https://github.com/jam1garner/smash-minecraft-skins/releases)

[My Twitter](https://twitter.com/jam1garner)


Thanks for reading!
