# Stuff I learned making this

This will probably always be a work-in-progress, but these are some notes about
how this all went, and what I gained in the process.

# First project using Rust

I've used rust before in some small learning environments, but it was much more
enlightening (and inspiring) to have something to ship as a final product (the
blog itself).
But for what it is, the blog is nothing near a CMS: it is basically 3 functions
right now that generate files in a pretty uninspiring directory layout.
But turns out, for what I need (a home page, a listing page, and the blog pages
themselves) that is more than enough.
For minor things I'm finding it incredibly easy to just add a function and call
it from main.
I am _absolutely_ certain that this isn't anywhere near a great design, but it
works and some abstractions, like the `Pages` struct are starting to appear.
Maybe next year (or whenever) I'll come back with more useage under my belt and
design something that I can look proudly upon.

## Rendering Markdown

This was a bit of a hassle, especially with there not quite being anything out
the box that covered my case: I need markdown parsed (easy enough), but each
file would have some metadata at the top that should _not_ be rendered.
So I had to reach for something lower level that would let me parse the top and
then mark where the real content began.
All in all, pattern matching made this a bit easier than I thought, and I
regained some confidence to say: "I know it doens't quite do this by itself, but
hey I can do it."

## `&str` and String

Man this one threw me for a loop, but at this point in the project (and with
some more reading) I'm finally getting used to lifetimes and using them
accordingly.
The difference between the two string types now makes much more sense and
actually helps me understand what's been hidden under the hood in Go.
In all honestly, while it was a pain at first, learning how the stack and heap
actually dealt with strings was much more rewarding and I enjoyt that extra bit
of control.

## Error handling

Man...I really liked what they did with errors in Go, and I've devoted a lot of
time to expressing the discernment and how to properly use them with my team.
But this type system...it's just incredible what I can express and encode into
the software.
The `Result` type alone is incredible, and it's zero-cost abstractions? (alway
has been).
Again, an error type isn't too much, just a short interface, but the type
conversions make this really ergonomic.

Plus the `WithMessage` trait gives me that `fmt.Errorf` context when I need it.
I was a little surprised that I could implement that on types that I _did not_
own.
I'd later find out that you either need to own the type or the trait in order to
do that, so it turns out there is a (reasonable) limit to this power.
And that you have to include the trait as a module import (I'm sure there's a
better name for ths), so I'm kinda okay with how explicit it is.
After all, the rest of the language is rather explicit.

## Modules

I started of Go-ing it by having just the one main file, but as that got kinda
tall and unweildy I broke it out.
I was definitely taken aback by the idea that a new file was a new module.
It makes more sense now, but it's definitely more knobs than I'm used to with
Go.
I'll be sure to try and figure out some conventions (from my own work as well)
from other projects to see what is the most convenient.

Also, I feel stupid not knowing `cargo install` can add stuff to the TOML for
your, but here we are.

## Rust Conclusion

I am definitely going to use some more rust.
I know it has limitations at some point, but the type system alone is getting me
excited for more and better API design in my programs.
I'll have some need to resize images for the blog, so maybe looking at making
this a multi-bin project and write tools for that as well.

# Blog design lessons

## I suck at design

I know the C.R.A.P. acronym, and have read books on print and design.
But then I get to the editor and I lose any semblance of taste and finesse.
So I started simple and added color from there.
It _seems_ to be the method that works for me, but this is an area where I'd
love to have something more foundational that points me in a direction on where
to start making flashier designs.
I mean look at [this guy](https://www.joshwcomeau.com/)!
This is his job (and clearly passion), but still: there's a lot about modern
front end that I need to keep up with.
