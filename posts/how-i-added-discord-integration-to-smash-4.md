<!--timestamp:1519966800-->

# How I added Discord Integration to Smash 4

If you’re just looking for a download link or pictures, see the very bottom of the post.

![](https://cdn-images-1.medium.com/max/2000/1*vMFfrE2r-NA9am5Btjo8CQ.png)

Discord has a feature called “Rich Presence” which allows games to add images, descriptions and timers to where it shows what you are currently playing. Developers can integrate this into their game and have it seamlessly show the user’s current in game status as their discord status (and in some cases it allows for joining or spectating, but that isn’t very practical for my purposes).

![](https://cdn-images-1.medium.com/max/2400/1*xH0gWNXHRAy6VqKuwSOzFg.png)

[I recently added support to Smash Forge](https://twitter.com/jam1garner/status/967939817945387008), an application of mine, due to a trend of users overwriting their game name with the current project they are working on and I saw as an opportunity to add an interesting feature. This had discord rich presence on my mind for the past couple days so I played around with the idea of adding it to other things before realizing I could possibly add it to Smash. So there are basically two routes I could go with this:

1. Reverse engineering the Discord RPC library and reimplementing it using only libraries available on the Wii U and integrating it as a mod for Smash (which would require further reverse engineering of the Smash binary)

1. Making a client for the user’s PC that communicates with the Wii U and interacting with the Discord RPC library

Sorry for anyone who was hoping this was going to be a reverse engineering blog post but option 2 sounds a hell of a lot easier. So, quick rundown on some relevant facts about the Wii U that helped me out.

* The Wii U doesn’t have ASLR (Address Space Layout Randomization) which means it doesn’t randomize the memory layout

* The Wii U always loads games to the same addresses every time, which means the RAM the game uses will have the same layout for any variables stored globally

If these weren’t the case it would likely require an inline code edit (basically instead of running our own code to just find the information we need in RAM, we’d have to modify the existing code to update our information whenever the value changes). So knowing this I hit up [Toad Stool](https://twitter.com/XAwesomeToadX), the person who (as far as I know) has the greatest experience with Smash 4 RAM hacks. Luckily he had all the addresses I needed so he saved me the tedious task of searching for the values in RAM and recording the addresses. They are as follows:

* Mode ID - 1098B2AB

* Stage ID - 1097577F

* Character IDs - 1098EDEB (P1), 1098EE6B (P2)

So now that I know where to get my information from memory, I need a server to run on the Wii U to allow me to grab these values. Originally I was planning on using a simple server that just waits to be polled, then returns the byte at the address however I ended up deciding against it as a more complex, existing program “TCPGecko” already was able to grab memory asynchronously from the game over TCP. So, since that already exists I now just needed to make a client to grab/handle this information from TCPGecko and I needed to setup some stuff for Discord integration to work. The =TCPGecko connection was rather simple as I gutted [TCPGecko dNET by Chadderz](https://github.com/Chadderz121/tcp-gecko-dotnet) in order to use it’s TCPGecko library as it’s not packaged separately as far as I am aware. That covers a good majority of the TCPGecko work, the only other thing I need is this snippet of code to grab the key values:

<iframe src="https://medium.com/media/5dce63be6129720f5e93dff156e0341d" frameborder=0></iframe>

Last thing I needed to do was set up the Discord integration portion. First up I created a new app for it under the discord development portal and grabbed the application id, which is basically just given to discord for identification. It lets it know what to call the game and what resources are available to it.

![](https://cdn-images-1.medium.com/max/2000/1*2MfvsI8LVXAoTkhXOha8hQ.png)

Another action that needed to be taken on the developer portal was to add any and all images the application needs for the status picture. In my case I wanted to display the stage as the picture for when they’re in game (I also uploaded the smash logo for the default image for menus and such, as well as the icon for stage builder for when they are using that). Given that I didn’t want to render every stage and deal with the annoyances of that (worth noting it wasn’t like I’d have to do this from scratch, Smash Forge has plenty of code for when we want to automate mass rendering of models) I ended up deciding to just convert over the existing game assets at my disposal. There are plenty of UI elements for every stage in smash, but I decided to go with stage_30 which are decent resolution square icons. Lucky for me fellow community member DSX8 had most of them (which are usually in NUT (Namco universal texture) format, which is somewhat similar to DDS) converted to PNG, and was kind enough to convert the last few over and send me the whole bunch, which was a great time saver. So I cleaned up the names, and they were just about ready to be uploaded.

![](https://cdn-images-1.medium.com/max/2078/1*kSX1FplprypAnwvs1YrhIw.png)

Problem is, they are at a resolution of 256x256 px rather than the 512x512 minimum discord requires. Frankly they look good enough that I didn’t mind having some below quality average so I just decided to upscale, sorry to anyone who would rather see me stick to the best practices. For this I just wrote a simple python script using pillow and glob to just upscale all PNGs in the folder.

<iframe src="https://medium.com/media/66146670189ddff06769fa3bc58199d7" frameborder=0></iframe>

Now for probably the worst part of all of this, uploading all the images to the discord developer portal and naming them. This was basically a process of click a button, navigate to the file, upload it, name it the internal name of smash (as that allow me to just name it what the image was called basically) and wait a few seconds after hitting upload. Wish I knew a faster way, wish the UX for uploading was better, but whatever now that I’ve uploaded over 50 images it’s too late for that. I don’t *like *discord’s image upload system for this especially compared with the rest of their great UI/UX designs, but I also understand it’s a bit ridiculous to sink too much time into the UX for a developer feature that maybe a couple hundred people will ever see.

![](https://cdn-images-1.medium.com/max/2000/1*QczNa16pWR9zBPBzzYnlGw.png)

Discord doesn’t really say they support C# but we can just use their Unity example, modify it a bit and it will wrap the DLLs for you. Then all I have to really do after that is write some code that interprets the mode and figures out what to display then passes that to the discord API. I needed ~400 lines of dictionaries for converting ids to names, ids to image keys, etc. but that was just a mixture of typing stuff in and having python scripts generate the dictionaries because man does typing out C# dictionaries get old. Last but not least the final result looks like this:

![](https://cdn-images-1.medium.com/max/2752/1*ThRb74nQYMXEYmjGiiD9Qg.png)

My Twitter:
[**jam1garner (@jam1garner) | Twitter**
*The latest Tweets from jam1garner (@jam1garner). Software Engineer, Console exploitation, Developer of Smash Forge and…*twitter.com](https://twitter.com/jam1garner)

Github Repo:
[**jam1garner/smash-disc4d**
*smash-disc4d - Discord rich presence integration for Smash 4 Wii U*github.com](https://github.com/jam1garner/smash-disc4d)

Download: [https://github.com/jam1garner/smash-disc4d](https://github.com/jam1garner/smash-disc4d)/releases
