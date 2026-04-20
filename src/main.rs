use std::f64::consts::PI;
use std::fs::File;
use std::io::{self, BufWriter, Write};

// ======== Vector math ========

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
    writeln!(w, "Ka 0.12 0.00 0.00")?;
    writeln!(w, "Kd 0.65 0.03 0.04")?;
    writeln!(w, "Ks 0.25 0.10 0.10")?;
    writeln!(w, "Ns 30.0")?;
    writeln!(w, "d 1.0")?;

    Ok(())
}

// ======== Petal generation ========

/// Paul Nylander's parametric rose surface (bugman123.com, 2009).
/// One continuous spiral sheet: phi (tilt) decays exponentially with theta,
/// so outer petals reflex while the inner spiral tightens into a bud.
/// x1 in [0,1] sweeps from spiral seam (origin) to free petal edge;
/// theta winds the spiral.
/// http://bugman123.com/Math/Rose.lsp
fn nylander_rose_point(x1: f64, theta: f64) -> [f64; 3] {
    let phi = 0.5 * PI * (-theta / (8.0 * PI)).exp();
    let y1 = 1.956_528_453_129_951
        * x1 * x1
        * (1.276_886_987_015_018_8 * x1 - 1.0).powi(2)
        * phi.sin();
    // Petal-edge wave: 3.6 ridges per 2π of theta, value in [0.5, 1.0].
    let m = (3.6 * theta).rem_euclid(2.0 * PI) / PI;
    let inner = 1.25 * (1.0 - m).powi(2) - 0.25;
    let x = 1.0 - 0.5 * inner.powi(2);
    let r = x * (x1 * phi.sin() + y1 * phi.cos());
    [
        r * theta.sin(),
        r * theta.cos(),
        x * (x1 * phi.cos() - y1 * phi.sin()),
    ]
}

fn generate_petals(mesh: &mut Mesh) {
    let theta1 = -20.0 * PI / 9.0;
    let theta2 = 15.0 * PI;
    let nu = 24;
    let nv = 575;
    let scale = 0.85;
    let z_offset = -0.05;

    let mut grid = vec![vec![[0.0f64; 3]; nv + 1]; nu + 1];
    for i in 0..=nu {
        let x1 = (i as f64) / (nu as f64) + 1.0e-6;
        for j in 0..=nv {
            let theta = theta1 + (theta2 - theta1) * (j as f64) / (nv as f64);
            let p = nylander_rose_point(x1, theta);
            grid[i][j] = [p[0] * scale, p[1] * scale, p[2] * scale + z_offset];
        }
    }
    mesh.add_grid("petal_inner", &grid);
}

// ======== Main ========

fn main() -> io::Result<()> {
    let mut mesh = Mesh::new();

    generate_petals(&mut mesh);

    mesh.write_obj("rose.obj")?;
    write_mtl("rose.mtl")?;

    let tri_count: usize = mesh.groups.iter().map(|g| g.triangles.len()).sum();
    eprintln!("Generated rose.obj and rose.mtl");
    eprintln!("{} vertices, {} triangles", mesh.vertices.len(), tri_count);

    Ok(())
}
