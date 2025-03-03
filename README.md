# 🦀🗣️ Crab Howler

This is where I experiment with sound synthesis by writing a very simple CLAP
plugin in Rust.

Follow along by reading the blog series:

- [Writing a CLAP synthesizer in Rust (Part 1)](http://kwarf.com/2024/07/writing-a-clap-synthesizer-in-rust-part-1/)  
Starting from nothing, lots of setup and boilerplate to create a simple
monophonic synthesizer without any UI or adjustable parameters.
- [Writing a CLAP synthesizer in Rust (Part 2)](http://kwarf.com/2024/07/writing-a-clap-synthesizer-in-rust-part-2/)  
This introduced parameters, specifically Attack, Decay, Sustain and Release in
order to create a "proper" ADSR envelope to fix clicking noises heard in the
first part. I also extend it to support 16 simultaneous voices, making it a
_polyphonic_ synth. Still no custom GUI, but the DAW is nice enough to provide
one for us for the parameters we expose.
- [Writing a CLAP synthesizer in Rust (Part 3)](https://kwarf.com/2025/03/writing-a-clap-synthesizer-in-rust-part-3/)
This added a custom GUI using [egui](egui.rs).
