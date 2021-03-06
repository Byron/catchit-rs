[![Build Status](https://travis-ci.org/Byron/catchit-rs.svg?branch=master)](https://travis-ci.org/Byron/catchit-rs)

This is a piston implementation of the similarly named [javascript implementation][catchit-js] *catchit*.

It runs in a window and requires freetype to be available on the system.

![Screenhot](https://raw.githubusercontent.com/Byron/catchit-rs/master/.screenshot.png)

# How to Run

After cloning, it should be as easy as ...

```bash
$ cargo run --release
```

... provided you have a stable rustc compiler and freetype installed.

**You will find pre-built binaries in the [Releases Section](https://github.com/Byron/catchit-rs/releases)**.

# Developer Diary

You can see exactly how the game was created by looking into episodes 1 to 15 within the [pistonics playlist][youtube-playlist] on *youtube*.

[\![thumb](https://i.ytimg.com/vi/r9qubF__YU4/mqdefault.jpg)][youtube-playlist]

# Credits

The implementation is based on *Robert Eisele's* 
[implementation][re-blog].

[catchit-js]: http://www.xarg.org/project/chrome-experiment/
[re-blog]: http://www.xarg.org/2010/02/my-very-first-chrome-experiment/
[youtube-playlist]: https://www.youtube.com/playlist?list=PLMHbQxe1e9MlsYw7WIQmqJusZbCkMl6dd