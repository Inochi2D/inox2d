# Inox2D

> An attempt at porting Inochi2D to Rust.

<p align="center">
  <img src="inox2d_logo.svg" />
</p>

How it came to be:

![origin](https://i.imgur.com/ZrTegpF.png)

## Implementation

Inox2D is currently a bit of a merge between [the original Inochi2D implementation](https://github.com/Inochi2D/inochi2d) in D and [Link Mauve's implementation](https://https://linkmauve.fr/dev/inochi2d/) in Rust (the [inochi2d](https://crates.io/crates/inochi2d) crate). The original is the standard, while Link's is a reverse-engineered and optimized implementation, but lacking the remaining core features that need to be implemented. His code is much simpler, but also not extensible (nodes are managed with an enum), so users of the library wouldn't be able to create custom nodes.

Inox2D is designed to be extensible. Nodes are extensible through the `Node` trait and users can create their own `NodeRenderer` with the `opengl` feature. That means a lot of cursed code involving `dyn Trait`, `Any` and trait downcasting. Hopefully in a future version of Rust I'll be able to make these abstractions cleaner with the [`Provider` API](https://rust-lang.github.io/rfcs/3192-dyno.html) (I prefer to stay on the stable version of Rust).

> Note: all the shader files that are present in this repository (under `shaders/`) have been copied from the original Inochi2D implementation, but are currently not used. Instead, it's using Link Mauve's simpler shaders.

## Optimization

| Implementation        | language | OpenGL calls |
| --------------------- | -------- | ------------ |
| Inochi2D standard     | D        | 3076         |
| Link Mauve's inochi2d | Rust     | 551          |
| Inox2D                | Rust     | 634          |

> To be honest, the lack of features is probably the major reason for the small number of calls.

## Status

The model is parsed and rendered correctly!

![parsing](https://0x0.st/onpz.png)

![foxgirl](https://0x0.st/on9F.png)

However, it's the only thing that works currently. It doesn't deform, doesn't have physics, and doesn't support animations.

- [x] Rendering (at least for the two example models)
- [ ] Deform
- [ ] Physics
- [ ] Animations

I *may* attempt to rewrite the OpenGL rendering code to be on-par with the standard Inochi2D implementation.

## License

This project is licensed under the 2-Clause BSD license.
See [LICENSE](LICENSE) for details.