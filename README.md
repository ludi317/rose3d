# rose3d

Parametric 3D rose generator. Outputs `rose.obj` + `rose.mtl`.

![rose](screenshot.png)

## Run

```sh
cargo run --release
open rose.obj
```

## Model

- 38 petals across 7 layers, spiraled by the golden angle
- Each petal is a curved sheet (numerically integrated centerline) with cupping, ruffling, and a center vein
- Petal bases attach along a parabolic bowl: `z = 0.12 - 1.5 r^2`
- Per-vertex smooth normals; petal width is clamped to its angular share to avoid clipping neighbors
- Receptacle, sepals, curved stem, and two compound leaves
