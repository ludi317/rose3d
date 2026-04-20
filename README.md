# rose3d

Parametric 3D rose generator. Outputs `rose.obj` + `rose.mtl`.

![rose](screenshot.png)

## Run

```sh
cargo run --release
open rose.obj
```

## Model

- Single continuous spiral surface — Paul Nylander's parametric rose
  ([bugman123.com](http://bugman123.com/Math/Rose.lsp), 2009), vendored
  at `bugman123.com/Math/Rose.lsp`
- φ (tilt) decays exponentially with θ, so outer petals reflex while the
  inner spiral tightens into a bud
- 25 × 576 grid sampled over `x1 ∈ [0,1]`, `θ ∈ [-20π/9, 15π]`
- Per-vertex smooth normals via finite differences
