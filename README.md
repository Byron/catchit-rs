[![Build Status](https://travis-ci.org/Byron/catchit-rs.svg?branch=master)](https://travis-ci.org/Byron/catchit-rs)

This is a piston implementation of the similarly named [javascript implementation][catchit-js] *catchit*.

It runs in a window and requires freetype to be available on the system.

# How to Run

After cloning, it should be as easy as ...

```bash
$ cargo run --release
```

... provided you have a stable rustc compiler and freetype installed.

# Credits

The implementation is based on *Robert Eisele's* 
[implementation][re-blog].

[catchit-js]: http://www.xarg.org/project/chrome-experiment/
[re-blog]: http://www.xarg.org/2010/02/my-very-first-chrome-experiment/