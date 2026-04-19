use std::f64::consts::PI;
use std::fs::File;
use std::io::{self, BufWriter, Write};

const TAU: f64 = 2.0 * PI;
const GOLDEN_ANGLE: f64 = 2.399_963_229_728_653;

// ======== Vector math ========

fn add3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize3(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

fn rotate_x(p: [f64; 3], a: f64) -> [f64; 3] {
    let (s, c) = a.sin_cos();
    [p[0], p[1] * c - p[2] * s, p[1] * s + p[2] * c]
}

fn rotate_z(p: [f64; 3], a: f64) -> [f64; 3] {
    let (s, c) = a.sin_cos();
    [p[0] * c - p[1] * s, p[0] * s + p[1] * c, p[2]]
}

fn pseudo_random(seed: u32) -> f64 {
    let x = seed.wrapping_mul(2654435761).wrapping_add(1013904223);
    (x % 10000) as f64 / 10000.0
}

// ======== Bowl geometry ========

/// Parabolic bowl surface: z height at radial distance r from center.
/// Inner petals sit high, outer petals sit lower on the bowl rim.
fn bowl_z(r: f64) -> f64 {
    0.12 - 1.5 * r * r
}

// ======== Mesh ========

struct Mesh {
    vertices: Vec<[f64; 3]>,
    normals: Vec<[f64; 3]>,
    groups: Vec<FaceGroup>,
}

struct FaceGroup {
    material: String,
    triangles: Vec<[usize; 3]>,
}

impl Mesh {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            groups: Vec::new(),
        }
    }

    fn add_grid(&mut self, material: &str, grid: &[Vec<[f64; 3]>]) {
        let rows = grid.len();
        let cols = grid[0].len();
        let base = self.vertices.len();

        for i in 0..rows {
            for j in 0..cols {
                self.vertices.push(grid[i][j]);

                let ip = i.saturating_sub(1);
                let in_ = (i + 1).min(rows - 1);
                let jp = j.saturating_sub(1);
                let jn = (j + 1).min(cols - 1);

                let du = sub3(grid[in_][j], grid[ip][j]);
                let dv = sub3(grid[i][jn], grid[i][jp]);
                self.normals.push(normalize3(cross3(dv, du)));
            }
        }

        let group = if let Some(g) = self.groups.iter_mut().find(|g| g.material == material) {
            g
        } else {
            self.groups.push(FaceGroup {
                material: material.to_string(),
                triangles: Vec::new(),
            });
            self.groups.last_mut().unwrap()
        };

        for i in 0..rows - 1 {
            for j in 0..cols - 1 {
                let v00 = base + i * cols + j;
                let v10 = base + (i + 1) * cols + j;
                let v11 = base + (i + 1) * cols + (j + 1);
                let v01 = base + i * cols + (j + 1);

                group.triangles.push([v00, v01, v11]);
                group.triangles.push([v00, v11, v10]);
            }
        }
    }

    fn write_obj(&self, path: &str) -> io::Result<()> {
        let f = File::create(path)?;
        let mut w = BufWriter::new(f);

        let tri_count: usize = self.groups.iter().map(|g| g.triangles.len()).sum();
        writeln!(w, "# Parametric 3D Rose")?;
        writeln!(
            w,
            "# {} vertices, {} normals, {} triangles",
            self.vertices.len(),
            self.normals.len(),
            tri_count
        )?;
        writeln!(w, "mtllib rose.mtl")?;
        writeln!(w)?;

        for v in &self.vertices {
            writeln!(w, "v {:.6} {:.6} {:.6}", v[0], v[1], v[2])?;
        }
        writeln!(w)?;

        for n in &self.normals {
            writeln!(w, "vn {:.6} {:.6} {:.6}", n[0], n[1], n[2])?;
        }
        writeln!(w)?;

        for group in &self.groups {
            writeln!(w, "usemtl {}", group.material)?;
            for t in &group.triangles {
                writeln!(
                    w,
                    "f {}//{} {}//{} {}//{}",
                    t[0] + 1,
                    t[0] + 1,
                    t[1] + 1,
                    t[1] + 1,
                    t[2] + 1,
                    t[2] + 1
                )?;
            }
            writeln!(w)?;
        }

        Ok(())
    }
}

fn write_mtl(path: &str) -> io::Result<()> {
    let f = File::create(path)?;
    let mut w = BufWriter::new(f);

    writeln!(w, "# Rose materials")?;
    writeln!(w)?;

    writeln!(w, "newmtl petal_inner")?;
    writeln!(w, "Ka 0.15 0.02 0.03")?;
    writeln!(w, "Kd 0.80 0.10 0.12")?;
    writeln!(w, "Ks 0.30 0.20 0.20")?;
    writeln!(w, "Ns 25.0")?;
    writeln!(w, "d 1.0")?;
    writeln!(w)?;

    writeln!(w, "newmtl petal_outer")?;
    writeln!(w, "Ka 0.12 0.01 0.01")?;
    writeln!(w, "Kd 0.65 0.04 0.05")?;
    writeln!(w, "Ks 0.25 0.15 0.15")?;
    writeln!(w, "Ns 20.0")?;
    writeln!(w, "d 1.0")?;
    writeln!(w)?;

    writeln!(w, "newmtl stem")?;
    writeln!(w, "Ka 0.02 0.06 0.02")?;
    writeln!(w, "Kd 0.12 0.38 0.12")?;
    writeln!(w, "Ks 0.10 0.10 0.10")?;
    writeln!(w, "Ns 10.0")?;
    writeln!(w, "d 1.0")?;
    writeln!(w)?;

    writeln!(w, "newmtl sepal")?;
    writeln!(w, "Ka 0.02 0.05 0.02")?;
    writeln!(w, "Kd 0.10 0.30 0.08")?;
    writeln!(w, "Ks 0.05 0.08 0.05")?;
    writeln!(w, "Ns 8.0")?;
    writeln!(w, "d 1.0")?;
    writeln!(w)?;

    writeln!(w, "newmtl leaf")?;
    writeln!(w, "Ka 0.02 0.06 0.02")?;
    writeln!(w, "Kd 0.15 0.45 0.10")?;
    writeln!(w, "Ks 0.10 0.15 0.10")?;
    writeln!(w, "Ns 15.0")?;
    writeln!(w, "d 1.0")?;

    Ok(())
}

// ======== Petal generation ========

struct PetalParams {
    tilt: f64,   // bend angle at petal tip (radians from vertical)
    curl: f64,   // extra tip curl via u^3
    length: f64, // arc length of petal centerline
    width: f64,  // half-width at widest point
    cup: f64,    // cupping intensity
    r_off: f64,  // radial offset: where petal base attaches on the bowl
    z_off: f64,  // vertical offset (computed from bowl_z)
    ruffle: f64, // edge ruffling amplitude
}

/// Integrate the petal centerline from 0 to u.
/// Returns (radial_distance, height, bend_angle_at_u).
fn centerline(u: f64, tilt: f64, curl: f64, length: f64) -> (f64, f64, f64) {
    let bend = tilt * u + curl * u.powi(3);
    if u < 1e-10 {
        return (0.0, 0.0, 0.0);
    }
    let steps = 50;
    let dt = u / steps as f64;
    let mut r = 0.0;
    let mut z = 0.0;
    for k in 0..steps {
        let t = (k as f64 + 0.5) * dt;
        let a = tilt * t + curl * t.powi(3);
        r += length * a.sin() * dt;
        z += length * a.cos() * dt;
    }
    (r, z, bend)
}

fn add_petal(mesh: &mut Mesh, p: &PetalParams, angle: f64, material: &str, seed: u32, n_in_layer: usize) {
    let nu = 24;
    let nv = 14;

    // Organic variation
    let size_var = 1.0 + 0.05 * (pseudo_random(seed) - 0.5);
    let tilt_var = p.tilt + 0.03 * (pseudo_random(seed + 1) - 0.5);
    let angle_var = angle + 0.03 * (pseudo_random(seed + 2) - 0.5);
    let r_off = p.r_off + 0.008 * (pseudo_random(seed + 3) - 0.5);

    let length = p.length * size_var;
    let width = p.width * size_var;
    let tilt = tilt_var;
    let curl = p.curl;

    let mut grid = vec![vec![[0.0f64; 3]; nv + 1]; nu + 1];

    for i in 0..=nu {
        let u = i as f64 / nu as f64;
        let (r_c, z_c, bend) = centerline(u, tilt, curl, length);

        // Current radial distance from rose center
        let r_total = r_off + r_c;

        // Limit petal width to avoid crossing same-layer neighbors.
        // Each petal gets an angular share of TAU / n_in_layer.
        // At radius r_total, max half-width ≈ r_total * sin(pi / n_in_layer).
        // Use 92% to leave a small gap.
        let max_hw = if r_total > 0.01 {
            r_total * (PI / n_in_layer as f64).sin() * 0.92
        } else {
            width // no limit near center
        };

        for j in 0..=nv {
            let s = -1.0 + 2.0 * j as f64 / nv as f64;

            // Width: narrow at base, wide toward top, rounded at tip
            let w_raw = width * u.sqrt() * (1.0 - u.powi(4)).sqrt();
            let w = w_raw.min(max_hw);
            let x = s * w;

            // Cupping perpendicular to petal surface
            let cup = p.cup * s * s * w * (1.0 - 0.5 * u);
            let y = r_off + r_c - cup * bend.cos();
            let z = z_c - cup * bend.sin();

            // Edge ruffling
            let ruffle = p.ruffle * s.abs().powi(3) * (8.0 * PI * u).sin();
            // Center vein
            let vein = 0.008 * (-8.0 * s * s).exp() * u * (1.0 - u);

            let pt = [x, y, z + ruffle + vein + p.z_off];
            grid[i][j] = rotate_z(pt, angle_var);
        }
    }

    mesh.add_grid(material, &grid);
}

fn generate_petals(mesh: &mut Mesh) {
    // r_off: radial attachment point on the bowl.
    // z_off: computed from bowl_z(r_off).
    // Each layer nests outside the previous one.
    let layers: Vec<(usize, PetalParams)> = vec![
        // Inner bud
        (3, PetalParams { tilt: 0.15, curl: 0.00, length: 0.50, width: 0.28, cup: 0.55, r_off: 0.00, z_off: bowl_z(0.00), ruffle: 0.000 }),
        (3, PetalParams { tilt: 0.30, curl: 0.03, length: 0.60, width: 0.32, cup: 0.45, r_off: 0.05, z_off: bowl_z(0.05), ruffle: 0.005 }),
        // Middle
        (5, PetalParams { tilt: 0.55, curl: 0.08, length: 0.75, width: 0.40, cup: 0.35, r_off: 0.10, z_off: bowl_z(0.10), ruffle: 0.010 }),
        (5, PetalParams { tilt: 0.80, curl: 0.15, length: 0.90, width: 0.46, cup: 0.25, r_off: 0.16, z_off: bowl_z(0.16), ruffle: 0.015 }),
        // Outer
        (7, PetalParams { tilt: 1.05, curl: 0.30, length: 1.05, width: 0.50, cup: 0.18, r_off: 0.23, z_off: bowl_z(0.23), ruffle: 0.020 }),
        (7, PetalParams { tilt: 1.25, curl: 0.50, length: 1.18, width: 0.52, cup: 0.12, r_off: 0.31, z_off: bowl_z(0.31), ruffle: 0.025 }),
        (8, PetalParams { tilt: 1.45, curl: 0.80, length: 1.30, width: 0.55, cup: 0.08, r_off: 0.39, z_off: bowl_z(0.39), ruffle: 0.030 }),
    ];

    let mut cumulative_angle = 0.0;
    let mut petal_id: u32 = 0;

    for (layer_idx, (n, params)) in layers.iter().enumerate() {
        let material = if layer_idx < 3 {
            "petal_inner"
        } else {
            "petal_outer"
        };
        for k in 0..*n {
            let angle = cumulative_angle + TAU * k as f64 / *n as f64;
            add_petal(mesh, params, angle, material, petal_id, *n);
            petal_id += 1;
        }
        cumulative_angle += GOLDEN_ANGLE;
    }
}

// ======== Sepals ========

fn generate_sepals(mesh: &mut Mesh) {
    let r = 0.42;
    let params = PetalParams {
        tilt: 2.0,
        curl: -0.05,
        length: 0.55,
        width: 0.12,
        cup: 0.05,
        r_off: r,
        z_off: bowl_z(r),
        ruffle: 0.0,
    };

    for i in 0..5u32 {
        let angle = TAU * i as f64 / 5.0 + 0.3;
        add_petal(mesh, &params, angle, "sepal", 100 + i, 5);
    }
}

// ======== Receptacle ========

fn generate_receptacle(mesh: &mut Mesh) {
    let steps_a = 24;
    let r_bowl = 0.43; // slightly beyond outermost petal r_off

    // Top: bowl surface (parabolic, concave up)
    {
        let steps_r = 12;
        let mut grid = vec![vec![[0.0f64; 3]; steps_a + 1]; steps_r + 1];
        for i in 0..=steps_r {
            let r = r_bowl * i as f64 / steps_r as f64;
            let z = bowl_z(r);
            for j in 0..=steps_a {
                let theta = TAU * j as f64 / steps_a as f64;
                grid[i][j] = [r * theta.cos(), r * theta.sin(), z];
            }
        }
        mesh.add_grid("stem", &grid);
    }

    // Underside: smooth taper from bowl rim down to stem
    {
        let z_rim = bowl_z(r_bowl);
        let r_stem = 0.047; // match stem top radius
        let z_stem = -0.25;  // match stem top z
        let taper_depth = z_rim - z_stem;
        let steps_h = 12;
        let mut grid = vec![vec![[0.0f64; 3]; steps_a + 1]; steps_h + 1];
        for i in 0..=steps_h {
            let t = i as f64 / steps_h as f64;
            // Smooth bulging taper
            let blend = (1.0 - t).powf(1.3);
            let r = r_stem + (r_bowl - r_stem) * blend;
            let z = z_rim - taper_depth * t;
            for j in 0..=steps_a {
                let theta = TAU * j as f64 / steps_a as f64;
                grid[i][j] = [r * theta.cos(), r * theta.sin(), z];
            }
        }
        mesh.add_grid("stem", &grid);
    }
}

// ======== Stem ========

fn stem_curve(t: f64) -> f64 {
    0.20 * (PI * t * 0.7).sin()
}

fn generate_stem(mesh: &mut Mesh) {
    let segments = 16;
    let vsteps = 30;
    let stem_height = 3.5;
    let mut grid = vec![vec![[0.0f64; 3]; segments + 1]; vsteps + 1];

    for i in 0..=vsteps {
        let t = i as f64 / vsteps as f64;
        let z = -0.25 - stem_height * t;
        let curve_x = stem_curve(t);
        let r = 0.035 + 0.012 * (1.0 - t);

        for j in 0..=segments {
            let theta = TAU * j as f64 / segments as f64;
            grid[i][j] = [curve_x + r * theta.cos(), r * theta.sin(), z];
        }
    }

    mesh.add_grid("stem", &grid);
}

// ======== Leaves ========

fn add_leaf(mesh: &mut Mesh, pos: [f64; 3], angle: f64, size: f64) {
    let nu = 14;
    let nv = 10;
    let mut grid = vec![vec![[0.0f64; 3]; nv + 1]; nu + 1];

    for i in 0..=nu {
        let u = i as f64 / nu as f64;
        for j in 0..=nv {
            let s = -1.0 + 2.0 * j as f64 / nv as f64;

            let serration = 1.0 - 0.1 * (10.0 * PI * u).sin().abs() * s.abs();
            let width_envelope = (PI * u).sin() * (1.0 - 0.3 * u);
            let w = size * 0.22 * width_envelope * serration;

            let lx = s * w;
            let ly = size * u;
            let lz = 0.015 * (3.0 * PI * u).sin() * s - 0.08 * u * u;
            let vein = 0.005 * (-6.0 * s * s).exp() * u * (1.0 - u);
            let lz = lz + vein;

            let tilted = rotate_x([lx, ly, lz], 1.4);
            let rotated = rotate_z(tilted, angle);
            grid[i][j] = add3(rotated, pos);
        }
    }

    mesh.add_grid("leaf", &grid);
}

fn generate_leaves(mesh: &mut Mesh) {
    let z1 = -1.3;
    let t1 = (-z1 - 0.25) / 3.5;
    let cx1 = stem_curve(t1);

    let z2 = -2.2;
    let t2 = (-z2 - 0.25) / 3.5;
    let cx2 = stem_curve(t2);

    add_leaf(mesh, [cx1, 0.0, z1], 0.3, 0.7);
    add_leaf(mesh, [cx1, 0.0, z1], 0.3 + 0.35, 0.5);
    add_leaf(mesh, [cx1, 0.0, z1], 0.3 - 0.35, 0.5);

    add_leaf(mesh, [cx2, 0.0, z2], PI + 0.5, 0.55);
    add_leaf(mesh, [cx2, 0.0, z2], PI + 0.5 + 0.35, 0.40);
    add_leaf(mesh, [cx2, 0.0, z2], PI + 0.5 - 0.35, 0.40);
}

// ======== Main ========

fn main() -> io::Result<()> {
    let mut mesh = Mesh::new();

    generate_petals(&mut mesh);
    generate_sepals(&mut mesh);
    generate_receptacle(&mut mesh);
    generate_stem(&mut mesh);
    generate_leaves(&mut mesh);

    mesh.write_obj("rose.obj")?;
    write_mtl("rose.mtl")?;

    let tri_count: usize = mesh.groups.iter().map(|g| g.triangles.len()).sum();
    eprintln!("Generated rose.obj and rose.mtl");
    eprintln!("{} vertices, {} triangles", mesh.vertices.len(), tri_count);

    Ok(())
}
