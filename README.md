<p align="center">
  <h1 align="center">Inox2D</h1>
  <p align="center">
    <img width="200" height="256" src="inox2d_logo.svg">
  </p>
  <div align="center">

Officially supported experimental Rust port of [Inochi2D](https://github.com/Inochi2D/inochi2d).
    &nbsp;
    <a align="center" href="https://discord.com/invite/abnxwN6r9v">
      <img align="center" src="https://img.shields.io/discord/855173611409506334?color=7289DA&label=%20&logo=discord&logoColor=white" alt="Discord" />
    </a>
  </div>
</p>

&nbsp;

The Inox2D workgroup provides support in the **#inox2d** channel on the [Inochi2D Discord][discord-invite].

**Currently this library and the specification is in a prototype state**, it is not recommended to use this library in production.

[discord-invite]: https://discord.com/invite/abnxwN6r9v

&nbsp;

## Rigging

If you're a model rigger you may want to check out [Inochi Creator](https://github.com/Inochi2D/inochi-creator), the official Inochi2D rigging app in development.  
This repository is purely for developers and is not useful if you're an end user.

&nbsp;

## Status

Both INP and INX files get parsed correctly.

Both renderers (OpenGL, WGPU) now work on all models we could test them on (Aka, Midori, Arch-chan).
The newer models which use the MeshGroup feature don't work yet though.

Support for mesh groups and animations is on the way!

### Feature tree

- [x] Parsing
  - [x] INP format
  - [x] INX format
- [x] Rendering
  - [x] OpenGL
    - [x] WASM (WebGL)
  - [x] WGPU
    - [ ] WASM (WebGL)
  - [ ] Draw List
- [x] Parameters
  - [x] Deforms (mesh vertex offsets)
  - [x] Values (node transform offsets)
  - [ ] Z-sort
- [x] Physics
- [ ] Animations
- [ ] Mesh groups

### INP and INX parsing

```sh
cargo run -p inox2d --features owo --example parse-inp path/to/puppet.inp
```

![Parsed foxgirl](https://fs.speykious.dev/inox2d/inox2d-parsed-foxgirl.png)

### OpenGL renderer

```sh
cargo run -p render-opengl path/to/puppet.inp
```

![OpenGL-rendered Arch-chan](https://fs.speykious.dev/inox2d/inox2d-opengl-arch-chan.png)

### WebGL demo

See the [`render_webgl`](/examples/render_webgl) example.

![WebGL-rendered Aka](https://fs.speykious.dev/inox2d/inox2d-webgl-foxgirl.png)

### WGPU renderer

```sh
cargo run -p render-wgpu path/to/puppet.inp
```

![WGPU-rendered Arch-chan](https://fs.speykious.dev/inox2d/inox2d-wgpu-arch-chan.png)

&nbsp;

## Implementation

Inox2D aims to support all features currently present in the standard D implementation.

Inox2D is designed to be extensible. Nodes are extensible through a generic `InoxData<T>` enum which has a `Custom(T)` variant. Every other part of the library accounts for it: the OpenGL renderer accepts any struct that implements the `CustomRenderer` trait to be able to render your custom nodes, and the deserialization functions accept generic `Fn`s for deserialization of custom nodes when it is relevant.

&nbsp;

## Optimization on OpenGL

| Implementation        | language | OpenGL calls |
| --------------------- | -------- | ------------ |
| Inochi2D reference*   | D        | 3076         |
| Link Mauve's inochi2d | Rust     | 551          |
| Inox2D                | Rust     | 1639         |

The OpenGL renderer on Inox2D has a few simple optimizations that result in fewer OpenGL calls:

- it uses a simple OpenGL cache to avoid making calls when the resulting state won't change,
- it only uploads a model's part textures once instead of every frame.

> \* Reference implementation is subject to change as optimisation passes are done, additionally code is more geared towards readability than performance for implementers to be able to more easily use it as reference.

&nbsp;

## License

This project is licensed under the 2-Clause BSD license.
See [LICENSE](LICENSE) for details.
