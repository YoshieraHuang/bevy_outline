# Bevy Outline Shader

A easy-use pixel-perfect outline shader for bevy. Inspired by [this wonderful tutorial](https://www.videopoetics.com/tutorials/pixel-perfect-outline-shaders-unity).

## Features

- [x] Pixel perfect: the width of drawn outline is almost the same as what we wanted.
- [ ] Eliminate foreshortening

## Usage

First, add `bevy_outline` as a dependency into your `Cargo.toml`:

```toml
[dependencies]
bevy_outline = "0.1.0"
```

Second, add `OutlinePlugin` into your app:

```rust
App::new()
    .add_plugin(OutlinePlugin)
```

Third, use `OutlineMaterial` as a mesh material:
```rust
commands
    .spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { ..default() })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_translation(translation),
        ..default()
    })
    .insert(outlines.add(OutlineMaterial {
        width: 5.,
        color: Color::rgba(0.2, 0.3, 0.4, 1.0),
    }));
```

## Problems

- [ ] the width of outliner seems not to be uniform.