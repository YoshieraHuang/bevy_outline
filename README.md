# Pixel-Perfect Outline Shader for Bevy

A easy-use pixel-perfect outline shader for bevy using vertex extrusion method. Inspired by [this wonderful tutorial](https://alexanderameye.github.io/notes/rendering-outlines).

## Features

- [x] Pixel perfect: the width of drawn outline is in pixel unit and the same as what we want.
- [x] Eliminate foreshortening: the width of outline is uniform from near view to far view.
- [x] Customizability. Width and color can be determined by user.  

## Usage

First, add `bevy_outline` as a dependency into your `Cargo.toml`:

```toml
[dependencies]
bevy_outline = "0.1.0"
```

Second, add `OutlinePlugin` into your app add set `Msaa` to a reasonable value:

```rust, norun
App::new()
    .insert_resource(Msaa { samples: 4})
... ...
    .add_plugin(OutlinePlugin)
... ...
```

Third, use `OutlineMaterial` as a mesh material:
```rust, norun
fn setup(
    ...
    mut outlines: ResMut<Assets<OutlineMaterial>>,
    ...
) {
    ...
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
    ...
}
```

Note that the unit of `width` attribute of `OutlineMaterial` is **pixel**.

## Demos

See [`Example`](https://github.com/YoshieraHuang/bevy_outline/tree/v0.1/examples)

## Problems

- [x] ~~the width of outliner seems not to be uniform.~~
- [ ] outline of built-in torus seems weird (algorithm is wrong and will be fixed in 0.8)

# Bevy Version Support

I intend to track the `main` branch of Bevy. PRs supporting this are welcome!

|bevy|bevy_outline|
|---|---|
|0.7|0.1|

# License

This project is licensed under the MIT License.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in bevy_outline by you, shall be licensed as MIT, without any additional terms or conditions.