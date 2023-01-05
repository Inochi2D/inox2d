<p align="center">
  <img width="256" height="256" src="inox2d_logo.svg">
</p>

# Inox2D

[![Discord](https://img.shields.io/discord/855173611409506334?color=7289DA&label=%20&logo=discord&logoColor=white)](https://discord.com/invite/abnxwN6r9v)

Officially supported experimental Rust port of [Inochi2D](https://github.com/Inochi2D/inochi2d). 

The Inox2D workgroup provides support in the **#inox2d** channel on the [Inochi2D Discord]().

**Currently this library and the specification is in a prototype state**, it is not recommended to use this library in production.

&nbsp;

## Rigging

If you're a model rigger you may want to check out [Inochi Creator](https://github.com/Inochi2D/inochi-creator), the official Inochi2D rigging app in development.  
This repository is purely for developers and is not useful if you're an end user.

&nbsp;

## Status

Due to licensing issues, **the library is currently in a broken state.**

A rewrite of the missing parts is on the way, particularly the OpenGL renderer, in order to be on-par with the standard Inochi2D implementation.

- [ ] Rendering
  - [ ] OpenGL
  - [ ] WGPU
- [ ] Deform
- [ ] Physics
- [ ] Animations

&nbsp;

## Implementation

Inox2D aims at supporting all features currently present in the standard D implementation.

Inox2D is designed to be extensible. Nodes are extensible through a generic `InoxData<T>` enum which has a `Custom(T)` variant. Every other part of the library accounts for it: the OpenGL renderer accepts any struct that implements the `CustomRenderer` trait to be able to render your custom nodes, and the deserialization functions accept generic `Fn`s for deserialization of custom nodes when it is relevant.

&nbsp;

## Optimization

| Implementation        | language | OpenGL calls |
| --------------------- | -------- | ------------ |
| Inochi2D reference*   | D        | 3076         |
| Link Mauve's inochi2d | Rust     | 551          |
| Inox2D                | Rust     | [TBD]        |

\* Reference implementation is subject to change as optimisation passes are done, additionally code is more geared towards readability than performance for implementers to be able to more easily use it as reference.

&nbsp;

## License

This project is licensed under the 2-Clause BSD license.
See [LICENSE](LICENSE) for details.
