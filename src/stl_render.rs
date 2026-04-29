use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use image::{ImageBuffer, Rgba, RgbaImage};

#[derive(Clone, Copy)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    fn sub(self, o: Self) -> Self { Self::new(self.x - o.x, self.y - o.y, self.z - o.z) }
    fn scale(self, s: f32) -> Self { Self::new(self.x * s, self.y * s, self.z * s) }
    fn dot(self, o: Self) -> f32 { self.x * o.x + self.y * o.y + self.z * o.z }
    fn cross(self, o: Self) -> Self {
        Self::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }
    fn length(self) -> f32 { self.dot(self).sqrt() }
    fn normalize(self) -> Self {
        let l = self.length();
        if l < 1e-6 { Self::new(0.0, 0.0, 0.0) } else { self.scale(1.0 / l) }
    }
}

/// A parsed STL mesh in memory. Cheap to render at different angles/sizes.
pub struct StlMesh {
    triangles: Vec<[Vec3; 3]>,
}

impl StlMesh {
    pub fn load(stl_path: &Path) -> Result<Self, String> {
        let mut file = std::fs::File::open(stl_path).map_err(|e| format!("Open STL: {}", e))?;
        let stl = stl_io::read_stl(&mut file).map_err(|e| format!("Parse STL: {}", e))?;
        let triangles: Vec<[Vec3; 3]> = stl.faces.iter().map(|f| {
            let v0 = stl.vertices[f.vertices[0]];
            let v1 = stl.vertices[f.vertices[1]];
            let v2 = stl.vertices[f.vertices[2]];
            [
                Vec3::new(v0[0], v0[1], v0[2]),
                Vec3::new(v1[0], v1[1], v1[2]),
                Vec3::new(v2[0], v2[1], v2[2]),
            ]
        }).collect();
        if triangles.is_empty() {
            return Err("STL has no faces".to_string());
        }
        Ok(Self { triangles })
    }

    /// Render to an RGBA image. Background is solid white. Flat Lambertian
    /// shading with a Z-buffer; backfaces are culled.
    /// `z_up=true` pre-rotates the model so its Z axis becomes screen-up
    /// (the convention used by most CAD / 3D-printing STLs).
    pub fn render(&self, size: u32, azimuth_deg: f32, elevation_deg: f32, z_up: bool) -> RgbaImage {
        let az = azimuth_deg.to_radians();
        let el = elevation_deg.to_radians();
        let (sa, ca) = (az.sin(), az.cos());
        let (se, ce) = (el.sin(), el.cos());
        // Pre-rotation for Z-up models: maps model (x, y, z) -> (x, z, -y),
        // i.e. the original Z axis points up in screen space.
        let pre_rotate = |p: Vec3| -> Vec3 {
            if z_up { Vec3::new(p.x, p.z, -p.y) } else { p }
        };
        let rotate = |p: Vec3| -> Vec3 {
            let p = pre_rotate(p);
            let p = Vec3::new(p.x * ca + p.z * sa, p.y, -p.x * sa + p.z * ca);
            Vec3::new(p.x, p.y * ce - p.z * se, p.y * se + p.z * ce)
        };

        let mut min = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        let rotated: Vec<[Vec3; 3]> = self.triangles.iter().map(|t| {
            let r = [rotate(t[0]), rotate(t[1]), rotate(t[2])];
            for v in &r {
                if v.x < min.x { min.x = v.x; }
                if v.y < min.y { min.y = v.y; }
                if v.z < min.z { min.z = v.z; }
                if v.x > max.x { max.x = v.x; }
                if v.y > max.y { max.y = v.y; }
                if v.z > max.z { max.z = v.z; }
            }
            r
        }).collect();

        let center = Vec3::new(
            (min.x + max.x) * 0.5,
            (min.y + max.y) * 0.5,
            (min.z + max.z) * 0.5,
        );
        let extent = (max.x - min.x).max(max.y - min.y).max(1e-6);
        let margin = 0.04_f32;
        let scale_xy = (1.0 - margin * 2.0) * size as f32 / extent;
        let cx = size as f32 * 0.5;
        let cy = size as f32 * 0.5;

        let project = |p: Vec3| -> (f32, f32, f32) {
            let dx = p.x - center.x;
            let dy = p.y - center.y;
            (cx + dx * scale_xy, cy - dy * scale_xy, p.z)
        };

        let light = Vec3::new(0.4, 0.7, 1.0).normalize();
        let base_color = (210u8, 140u8, 70u8);

        let w = size as i32;
        let h = size as i32;
        let mut z_buf = vec![f32::NEG_INFINITY; (w * h) as usize];
        let mut img: RgbaImage = ImageBuffer::from_pixel(size, size, Rgba([255, 255, 255, 255]));

        for tri in &rotated {
            let normal = tri[1].sub(tri[0]).cross(tri[2].sub(tri[0])).normalize();
            if normal.z < 0.0 { continue; }
            let intensity = (normal.dot(light) * 0.65 + 0.35).clamp(0.18, 1.0);
            let r = (base_color.0 as f32 * intensity) as u8;
            let g = (base_color.1 as f32 * intensity) as u8;
            let b = (base_color.2 as f32 * intensity) as u8;
            let color = Rgba([r, g, b, 255]);

            let (x0, y0, z0) = project(tri[0]);
            let (x1, y1, z1) = project(tri[1]);
            let (x2, y2, z2) = project(tri[2]);
            rasterize_triangle(&mut img, &mut z_buf, w, h, x0, y0, z0, x1, y1, z1, x2, y2, z2, color);
        }

        img
    }
}

/// Convert a github.com/.../blob/... URL to its raw.githubusercontent.com equivalent.
fn github_blob_to_raw(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("https://github.com/") {
        if let Some(idx) = rest.find("/blob/") {
            let user_repo = &rest[..idx];
            let path = &rest[idx + "/blob/".len()..];
            return format!("https://raw.githubusercontent.com/{}/{}", user_repo, path);
        }
    }
    url.to_string()
}

/// Resolve an STL input (local path or http(s) URL) to a local file path,
/// downloading to a tempfile if needed.
pub fn fetch_stl(input: &str) -> Result<PathBuf, String> {
    if input.starts_with("http://") || input.starts_with("https://") {
        let url = github_blob_to_raw(input);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        let resp = client.get(&url).send().map_err(|e| format!("Download: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Download failed: HTTP {}", resp.status()));
        }
        let bytes = resp.bytes().map_err(|e| format!("Read body: {}", e))?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let temp_path = std::env::temp_dir().join(format!("storetemplate_stl_{}.stl", timestamp));
        std::fs::write(&temp_path, &bytes).map_err(|e| format!("Write tempfile: {}", e))?;
        Ok(temp_path)
    } else {
        let p = PathBuf::from(input);
        if !p.exists() {
            return Err(format!("File not found: {}", p.display()));
        }
        Ok(p)
    }
}

/// Spawn a background thread that fetches an STL (path or URL) and parses it.
/// The receiver yields one message: `Ok(StlMesh)` or `Err(message)`.
pub fn load_stl_async(input: &str) -> mpsc::Receiver<Result<StlMesh, String>> {
    let (tx, rx) = mpsc::channel();
    let input = input.to_string();
    thread::spawn(move || {
        let path = match fetch_stl(&input) {
            Ok(p) => p,
            Err(e) => { let _ = tx.send(Err(e)); return; }
        };
        let _ = tx.send(StlMesh::load(&path));
    });
    rx
}

/// Convenience: load an STL and render it to a PNG file in one call.
pub fn render_stl_to_png(
    stl_path: &Path,
    output_path: &Path,
    size: u32,
    azimuth_deg: f32,
    elevation_deg: f32,
    z_up: bool,
) -> Result<(), String> {
    let mesh = StlMesh::load(stl_path)?;
    let img = mesh.render(size, azimuth_deg, elevation_deg, z_up);
    img.save(output_path).map_err(|e| format!("Save PNG: {}", e))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn rasterize_triangle(
    img: &mut RgbaImage,
    z_buf: &mut [f32],
    w: i32,
    h: i32,
    x0: f32, y0: f32, z0: f32,
    x1: f32, y1: f32, z1: f32,
    x2: f32, y2: f32, z2: f32,
    color: Rgba<u8>,
) {
    let min_x = x0.min(x1).min(x2).floor().max(0.0) as i32;
    let max_x = x0.max(x1).max(x2).ceil().min(w as f32 - 1.0) as i32;
    let min_y = y0.min(y1).min(y2).floor().max(0.0) as i32;
    let max_y = y0.max(y1).max(y2).ceil().min(h as f32 - 1.0) as i32;

    let denom = (y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2);
    if denom.abs() < 1e-6 { return; }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let l0 = ((y1 - y2) * (px - x2) + (x2 - x1) * (py - y2)) / denom;
            let l1 = ((y2 - y0) * (px - x2) + (x0 - x2) * (py - y2)) / denom;
            let l2 = 1.0 - l0 - l1;
            if l0 < 0.0 || l1 < 0.0 || l2 < 0.0 { continue; }
            let z = l0 * z0 + l1 * z1 + l2 * z2;
            let idx = (y * w + x) as usize;
            if z > z_buf[idx] {
                z_buf[idx] = z;
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}
