sprite_anim
==========================================================================
Sprite and animation implementations for [egui](https://github.com/emilk/egui).

# How to use
add lines below to your **Cargo.toml**

```
sprite_anim = "1.0.0"
```

# Feature Flags

This crate provides the features listed below for minimizing dependencies and compilation time.

| Feature | Details | Default |
| :--- | :--- | :--- |
| `gif_player` | Provide gif image loading and play animation feature | **Yes** |
| `sprite` | Provide sprite sheet loading feature  | **Yes** |
| `sprite_animation` | Provide sprite animation loading feature | **Yes** |

### Example: Use only `sprite` features
Add lines below to `Cargo.toml`

```toml
[dependencies]
my_library = { version = "1.0.0", default-features = false, features = ["sprite"] }
```

# Example App
You can find full working example in **example/main.rs**.
The example can be built and run with command below.

```
cargo build --example main
cargo run --example main
```

# Asset Source
The example app contains the images downloaded from the links below.
- [Pixel stickman running gif.gif](https://commons.wikimedia.org/wiki/File:Pixel_stickman_running_gif.gif)
- [Perfect-loop-cube.gif](https://commons.wikimedia.org/wiki/File:Perfect-loop-cube.gif)
- [8-cell.gif](https://commons.wikimedia.org/wiki/File:8-cell.gif)
- [Platonic Solids Stereo 5 - Icosahedron.gif](https://commons.wikimedia.org/wiki/File:Platonic_Solids_Stereo_5_-_Icosahedron.gif)

I show my gratitude and respect for [Wikimedia Commons](https://commons.wikimedia.org/wiki/Main_Page) and its contributors.
