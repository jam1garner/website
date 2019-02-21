
# Making a Decompiler from Nothing — A Postmortem



I have recently had the not-so-common challenge of working with a completely undocumented interpreter. With this opportunity I got to research and develop tools for it from scratch and with that comes some interesting challenges. Originally I started with an assembler and disassembler, but convincing people who are barely familiar with code (if at all) to reverse engineer and edit thousands of unlabeled files written in an assembly language made up by a hobbyist reverse engineer is… a hard sell to say the least. So the logical next step is to abstract it, the best choice of which being decompilation to an existing language. Using an existing language makes teaching easier as those familiar with it already understand the syntax and those who aren’t familiar with it have resources to learn from.

### The Basics of Decompilation

Decompilation is the art of abstraction, taking simple parts and turning them into a bigger idea. Think of it like a sentence compared to its meaning; a sentence is just a bunch of words, each word has it’s own simple meaning that can come together to describe precisely what is happening.

![](https://cdn-images-1.medium.com/max/4200/1*CrCo-zPWu-LRo_xt6o1WsA.png)

If we look at this analogy we can see where some of the difficulties of decompilation come from as order affects the meaning, words can be strung together in different combinations to have completely different meanings, and that to understand the whole idea we have to understand how the parts interact with each other. If the idea of looking at words to figure out the meaning of a sentence sounds a bit daunting, don’t worry as the analogy of [natural language processing](https://en.wikipedia.org/wiki/Natural_language_processing) is quite a bit harder than decompilation itself.

### Going from Understanding to Implementation

For starters you, at the very least, need a basic understanding of how different structures in your language of choice could be replicated in whatever architecture you’re working with. Asking questions like “what would an if statement look like”, “what is a loop made of in [architecture]”, and “how are variables handled within the architecture” are things you need answers to before you even start designing your decompiler. Let’s take a simple example with a C if statement and how it would be compiled to an imaginary architecture.

    // C

    if(var0 < 0){
        var0 = 0;
    }

    ---------------------------

    ; imaginaryAsm

    compareLessThan var0, 0
    jumpIfNotTrue EndLabel
    set var0, 0
    EndLabel:

Important takeaways:

* Any statement can be used for the condition

* Any number of statements can be inside the if statement

This raises a problem, if any combination of statements can be used in an if statements, we need to somehow generalize our logic so that it doesn’t matter what the condition is and it doesn’t matter what statements are inside the if statements. In essence we need treat it almost like a template

    [condition]
    jumpIfNotTrue EndLabel
    [statements]
    EndLabel:

    -----------------------------

    if([condition]){
        [statements]
    }

This way we can recursively decompile the “bigger” structures and, rather than handle every possible type of if statement, we just decompile the condition and inner statements separately (as well as recursively) and insert them into their respective positions in the if statement.

### Knowing your Enemy

To understand decompilation you first have to understand compilation as to get any meaningful decompilation results on many architectures you need to understand the compiler that was used on the targeted file. While both GCC and Clang are C compilers that implement the same standards, the resulting executables won’t be identical due to there being more than one way implement variables/control flow, optimize outputs, or they may even being using different implementations of the C standard library. Because of this, you need to understand the nuances of the compiler that was used for the files you’re targeting in order to understand what you’re looking for.

There are plenty of ways to work out how each compiler works, even if you don’t have the compiler itself. If you can track down the compiler (or better yet its source) you can use that to find out how it works. However if, like me, you’re not a big fan of trying to familiarize yourself enough with a compiler’s inner-workings, you can either make or find examples of the compiler’s output in order to hand decompile the code. Hand decompilation is a pretty simple process once you get the basics of the assembly language you’re working with, just:

1. Read the code you want to decompile

1. Understand how it works and what it means

1. Reimplement it in C

While this is overly laborious and takes a bit of practice, this should come to anyone with decent reverse engineering experience naturally. If you have trouble understanding how exactly a bit of code works, it helps to come up with some scenarios and figure out how exactly the code would handle it. If you see a variable X being compared with 5, consider what the code will do if X is less than, equal to or greater than 5 respectively. While there is no real formula to reverse engineering how code works, practice will help immensely as you’ll start to recognize patterns and figure out faster ways of understanding different forms of control flow.

### Getting into the Specifics

While I’ve tried to keep most of my advice and thoughts general enough that it could be applied to any decompilation project, a lot of decompilation is specific to the architecture. While architectures like ARM and x86 share a lot of similarities and can use very similar decompilation techniques because of it, these techniques may not be applicable to other architectures such as PowerPC or something even more proprietary. In my case, I wrote a decompiler for something called **M**otion**SC**ript or MSC for short, which is a proprietary architecture designed for and used in the videogame Super Smash Brothers for Wii U for the logic that controls characters. It, unlike any real architectures I’ve ever worked with, is a stack based architecture that passes values to other commands by pushing them onto a stack to be popped off. The MSC architecture has its benefits being rather simple to determine how and where values are used as they can only be used by the next command that pops off the stack. The downside of this is that a non-standard architecture can’t really be based off of standard methods. If you’re writing a decompiler your attack strategy is going to be dependent on the unique design of the architecture you’re working with, there’s no silver bullet.

### Conclusion

Writing a decompiler is definitely one of the most fun experiences I’ve had with reverse engineering. It’s a great learning experience, helps improve your understanding of the abstractions of programming languages and gives rather satisfying results once you begin to succeed. If you’re interested in further reading on understanding the abstractions of computers, I’d highly recommend [Cody Brocious](https://twitter.com/daeken)’ slides, [“Becoming a full-stack reverse engineer.”](https://twitter.com/daeken/status/1036081396157239297)

Have questions? Interested in my other projects? [DM and/or follow me on twitter](https://www.twitter.com/jam1garner).
