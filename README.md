# rplace-2023

r/place realtime player for 2023, using [Vulkano](https://github.com/vulkano-rs/vulkano).

Inspired by [Applying 5 million pixel updates per second with Rust & wgpu](https://maxisom.me/posts/applying-5-million-pixel-updates-per-second).

![Screenshot](<images/Screenshot 2023-12-01 at 10.42.37.png>)

Compute shader is used to update the canvas, resulting in 10000x playback speed with ocassional frame drops.

`bincode` & gz are used to compress and read the pixel updates data.

## Todos

- [x] Color space correction.
- [ ] Trialing data should be processed.
- [ ] Player control.
