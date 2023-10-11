---

title: Everything You Never Wanted to Know About Go Errors
publishDate: 2023-07-31
excerpt: Understanding modern Go errors by starting at the beginning
slug: everything-go-errors

---

What's there to say about Go's error handling?
Can't we just
```go
if err != nil {
    return err
}
```
and call it a day?
And for many outsiders to the language, that bit of meme might be the only exposure
they've had.
But over the years, it has gotten a bit more robust, in its own way with its own
idioms.

What follows is close to a brain dump of everything I know/think about Go's error handling.
Meaning: this is certainly less of a technical resource, but hopefully my years of trying
to work with it results in some insight.
Unfortuntely, to best understand Go's version of errors, I think we have to look at
how it was first introduced and the history up til now.

# The built-in `error` interface

Go did not always have `error`.
Prior to the Go 1 release, it had a concept of errors, but no unified interface for them.
Back in 2011, the `os` package defined its own `Error` interface:
```go
package os

// An Error can represent any printable error condition.
type Error interface {
	String() string
}

// PathError records an error and the operation and file path that caused it.
type PathError struct {
	Op    string
	Path  string
	Error Error
}

func (e *PathError) String() string { return e.Op + " " + e.Path + ": " + e.Error.String() }
```

And functions returned this interface with signatures like these:
```go
func (p *Process) Wait(options int) (w *Waitmsg, err Error) {
    // Omitted...
}

func (p *Process) Signal(sig Signal) Error {
    // Omitted...
}
```

Not too far off from what you see today, huh?
More interestingly, it seems the rest of the go standard lib used this interface as well,
even if wasn't a particularly `os`-adjacent package:
```go
func Scanf(format string, a ...interface{}) (n int, err os.Error) {
	// Omitted...
}
```

Finally, in [Go 1's release notes](https://go.dev/doc/go1), a new built-in gets a nice,
big [section](https://go.dev/doc/go1#error).
It gives a definition for the `error` interface we know and love:
```go
type error interface {
    Error() string
}
```
It's pretty close to what was in the `os` package, and there's another section that mentions
how this replaces that type and its effect on every other dependent package.
To use it, take advantage of duck-typing, and implement `Error() string` on whatever error type you have.

Okay, so we've got an interface! It's now built-in and official!
But let's say we just called a function that returns a value and an `error`.
So how are we using it?

## Just return it

Mentioned at the very beginning, the prevalent recourse here was to just return it and 
exit our current function:
```go
rune, err := reader.ReadRune()
if err != nil {
    // Something bad happened, just exit
    return err
}
```
And then up above, where we decide to terminate our program because of the error,
you could do a number of things, but most look like this:
```go
val, err := myFunction()
if err != nil {
    panic(err.Error())
}
```
Or `log.Fatal` or `fmt.Println` and `exit`, take your pick.
But what if we wanted to handle it a bit more, instead of just bubbling up to `main`?

## Check for `Sentinel` errors

I'm stealing some terminolgy from Dave Cheney here, and his [presentation/article](https://dave.cheney.net/2016/04/27/dont-just-check-errors-handle-them-gracefully)
on gracefully handling errors.
The idea of a _sentinel_ error is that there's some _static_ value in a package that signifies
what error was just returned.
For example, in the `io` package defines one:
```go
// EOF is the error returned by Read when no more input is available.
// Functions should return EOF only to signal a graceful end of input.
// If the EOF occurs unexpectedly in a structured data stream,
// the appropriate error is either ErrUnexpectedEOF or some other error
// giving more detail.
var EOF = errors.New("EOF")
```
Unforunately, we have to use `var` here instead of `const`, but it's heavily implied your shouldn't change this value.
Another note is that this error is constructed with `errors.New`, which returns a type that implements
`error` backed by a string you give it.

Using this error would mean checking for equality when we're handling it:
```go
rune, err := reader.ReadRune()
if err == io.EOF {
    // This is an OK error! We just ran out of stuff to process
    return s, nil
}
if err != nil {
    // Something bad happened, just exit
    return "", err
}
```
Now functions can kinda tell each other what happened and we change behavior based on it.
But, this relies on the error being returned exactly the reference to the error defined.
So what about a more detailed error, something more complicated?

## Type casting

Not all errors are backed by a single string, and they have fields that vary.
For example, this one:
```go
// A ParseError represents a malformed text string and the type of string that was expected.
type ParseError struct {
	Type string
	Text string
}

func (e *ParseError) Error() string {
	return "invalid " + e.Type + ": " + e.Text
}
```
It's belongs to the `net` package in `ip.go`, and I'll note that error structs seem to be the exception,
rather than the rule, in the early Go std lib.
It has a few fields that describe different parts of what went wrong, specifically a `Type`, which might tell
us a bit more about what happened, and a `Text`, which tells us the piece that went wrong.
And attached to it is a func that makes this type a valid implementation of `error`.

Go doesn't give us too many tools to handle this, but assuming we wanted to deal with the innards of this error,
we'd have to do the following:
```go
val, err := someFunc()
if parseErr, ok := err.(*ParseError) {
    if parseErr.Type = "someType" {
        // Do a thing because of the type
    } 
}
if err != nil {
    // Can't handle this, bubble up
    return err
}
```

Cheney has a bunch of remarks on how this affects the API of your package and architecture between them,
but the one thing I'll point out is that we have to _know_ what errors to check for.
Not a huge problem, and not to get ahead of myself, but it's still an issue, and something to keep in mind.
Everyday useage of this might include casting an `error` to a `mysql.Error` so that you can read the error code
and due something because of it, e.g. returning a conflict status when we've tripped a unique constraint in
our database.

## String checking

I do not condone this method at all, use at your own risk.
Furthermore, if you're having to do this, I suggest reworking the code so you don't have to.

For our final method, it involves comparing the `Error()` string that every `error` has.
It might look something like this:
```go
val, err := someFunc()
if err != nil && strings.Contains(err.Error(), "CONFLCIT") {
    // Hey we got a conflict, do something about it
}
```

But please don't do it.
Among a long list of reasons, maybe the best one is that you know _WAY_ too much
about the innards of the error and it's likely to change, making this flaky as all hell.

# Go 1.13

I know we started with Go 1, and now the heading reads Go 1.13, but not much happens with error handling
in the (7) years between releases in terms of errors.
Sure, a bunch of packages got new error types and changed their API's with regards to what errors they emitted,
but nothing particularly of note to the average Go user.

(Movie announcer voice) _but then_... a 
