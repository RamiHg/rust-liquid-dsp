# rust-liquid-dsp

This is a Rust wrapper for [liquid-dsp](https://github.com/jgaeddert/liquid-dsp). It does not rely
on autotools, and compiles the library purely within Rust. I've also created a simple wrapper
interface to make it more ergonomic to use with Rust, and to add memory safety.

However, I've only configured the build script to compile and generate bindings for the filter
module, which I'm using to resample audio in my [other project](https://github.com/RamiHg/RustyBoy).

I'd be happy wrap the rest of the library if there is enough interest. Open an issue or pull request
if you're interested!