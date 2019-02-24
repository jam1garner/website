<!--timestamp:1521000000-->
<!--![](https://miro.medium.com/max/782/1*uhDB-QPDEO-_Tl-Fs0IVvQ.png)-->

# How I turned my DK Bongos Into a Keyboard

AS YOU MAY ALREADY KNOW THE DK BONGOS ARE A SPECIAL GAMECUBE CONTROLLER MADE FOR “DONKEY KONGA 1/2” AND “DK JUNGLE BEAT”.

If you’re wondering why I wrote that in all caps it’s because I wrote it using my DK Bongos and I currently don’t support lowercase. If you’re wondering why I spent 30 minutes typing a sentence using bongos it’s because that’s likely the level of masochism you were looking for from me when you decided to spend a couple minutes of your life reading about using Nintendo’s bongo themed controller as a keyboard. So, if you happen to follow me on twitter, you probably have already seen my musical keyboard and this x3 speed 30 second clip of me typing using it:

<blockquote class="twitter-tweet" data-lang="en"><p lang="tl" dir="ltr">DK Bongo typing in action. <a href="https://t.co/0wxlE4Abuw">pic.twitter.com/0wxlE4Abuw</a></p>&mdash; jam1garner (@jam1garner) <a href="https://twitter.com/jam1garner/status/973667327861456897?ref_src=twsrc%5Etfw">March 13, 2018</a></blockquote>
<script async src="https://platform.twitter.com/widgets.js" charset="utf-8"></script>

And, since this is my latest “dumb but cool stuff I spent 24 hours straight writing” project I figured it deserved a blog post, especially since I feel I learned a decent bit worth sharing. Here’s a basic rundown of how it works from a higher level:

![](https://cdn-images-1.medium.com/max/2000/1*aIn-J_TDuDyuPlplDBvhKw.png)

This is a bit simplified of course, but shows more or less how everything is communicating.

The Mayflash gamecube adapter has a “PC” mode which, instead of using the communication style of Nintendo’s gamecube adapter for Wii U, allows you to use it as an HID adapter. If you aren’t familiar with HID, it’s a class of device for USB that made for user input (Unabbreviated HID is “Human Interface Device”). This is used for anything from keyboards to controllers. The basic style of communication is called “polling” which is basically checking every so often for the state of the controller. For this project I used HID library for C#, which I’ve found works quite well (my only issue being that the sample code polls the controller constantly and eats up all your CPU, so you have to tell it to slow down a bit). The process is pretty simple once you understand it and can be broken down to a couple of steps:

1. Check if the controller is plugged in by looking for the Vendor ID and Product ID of the Mayflash adapter (which has a Vendor ID of 0x0079 and a Product ID of 0x1844) and if so open the device through HID library and set events for when it’s inserted and removed.

1. When it is inserted (which, if plugged in correctly, should happen immediately) it will run the corresponding event. All this does is log the controller is connected to console and then tell it to poll for a message from the controller and to run the OnReport event when it gets a message.

1. For every time it polls the controller it receives a message which it parses to figure out the corresponding button presses a mic level. (I also check that the bongos are inserted so I can print a message if it isn’t connected to the right port).

Message parsing is somewhat tricky, especially since I wasn’t familiar with it in anyway before now, so I just had it print the message to console while I pressed buttons in order to figure it out. Some example messages for when I clap with no buttons held look like:

<script src="https://gist.github.com/jam1garner/007774eb252c0951c7aecf96531ba4f6.js"></script>

So some notable features:

* The FFFF is constant no matter what buttons you press but is 0000 if you don’t have a controller plugged in, this is an easy way to tell if the bongos are in the right port.

* The eighth byte (starts as 0A, goes up to 6F then slowly goes back to 0A) is the microphone detected volume, I just check if it is higher than a desired threshold (I used 0x40, but I still sometimes have trouble clapping loud enough, just picked it as a guess to give a bit of wiggle room for claps quieter than my 6F).

For the buttons it is just the first 2 bytes as a bitfield with the layout:

    Byte 1:

    1 bit — Top right button

    2 bit — Bottom right button

    4 bit — Bottom left button

    8 bit — Top left button

    Byte 2:

    2 bit — Start button

So for example if I am holding just the top right button it might look something like this:

    010000FFFF00000A08

And if I hold both the top left and bottom left it will look like:

    0C0000FFFF00000A08

After parsing the inputs, I compare them to the last frame to see if this is the first frame a button is pressed, as I don’t want holding a button (or heaven forbid, pressing a button for more than one time polled!) to do the input multiple times. If you disagree with taking away the ability to hold down a button to do things multiple times, feel free to shoot me a tweet showing a beat you make my just holding down the bongos, I’m not some heathen that’d make a inaccurate bongo keyboard so please take your “smart design” elsewhere. Lastly I have logic for turning these inputs into keypresses, most of which isn’t very special. The keypress code is extremely simple:

<script src="https://gist.github.com/jam1garner/a5220690bbbbc0676e421f6896d5afed.js"></script>

And the way I “cycle” through characters is essentially hit backspace and type the new character at the same time.

Want more stuff like this? Check out my other posts or follow me on twitter:
[**jam1garner (@jam1garner) | Twitter**
*The latest Tweets from jam1garner (@jam1garner). Software Engineer, Console exploitation, Developer of Smash Forge and…*twitter.com](https://twitter.com/jam1garner)

Want to try out the DK Bongo Keyboard yourself? (Note: requires Mayflash adapter and DK Bongo) Grab the latest release here:
[**jam1garner/dk-bongo-keyboard**
*dk-bongo-keyboard - Application for interfacing with DK Bongos over HID for use as a keyboard*github.com](https://github.com/jam1garner/dk-bongo-keyboard/releases)

Want a more in depth look or instructions on how to use? Check out the github repository itself (Instructions in readme):
[**jam1garner/dk-bongo-keyboard**
*dk-bongo-keyboard - Application for interfacing with DK Bongos over HID for use as a keyboard*github.com](https://github.com/jam1garner/dk-bongo-keyboard)
