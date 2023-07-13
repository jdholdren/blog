---

title: Everything Go Errors
publishDate: 2023-07-31
excerpt: Everything you didn't want to know about Go errors.
slug: everything-go-errors

---

What's there to say about Go's error handling?
Can't we just `if err != nil { return err }`?
And for many outsiders to the language, that bit of meme might be the only exposure
they've had.
It's not an entirely bad understanding, either, but as we near Go 1.21, it's incomplete.

What follows is closer to a brain dump of everything I know/think about Go's error handling.
Meaning: this is certainly less of a technical resource, but hopefully my years of trying
to do it in a more designed and maintainable manner results in some insight.
