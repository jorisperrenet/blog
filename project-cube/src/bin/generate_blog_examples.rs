//! Generates every reproducible asset under `blog-svelte/static/assets/`:
//! - tower interactives (`tower_*.html` and `pieces_overview.html`)
//! - card silhouettes as inline SVGs
//! - a fresh copy of `compat_graph.html`
//!
//! Run with `cargo run --release --bin generate_blog_examples`.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufRead, BufReader};

use project_cube::balance::{check_stability_quick, Placement};

const ASSETS_DIR: &str = "../static/project-cube/assets";

// Default game masses (per cell): full = 150 g, hollow = 50 g  (=  3 : 1).
const FULL_CELL_MASS: f64 = 150.0;
const HOLLOW_CELL_MASS: f64 = 50.0;

// ---------------------- examples manifest ----------------------------------
// `save_marked` keeps writing static HTML for backward compat / direct use,
// but the blog post itself iframes a generic `tower_view.html?example=NAME`
// page that fetches `examples_manifest.json` to look up the placement data.

use std::cell::RefCell;
thread_local! {
    static MANIFEST: RefCell<Vec<(String, ManifestEntry)>> = RefCell::new(Vec::new());
}
struct ManifestEntry {
    label: String,
    front: u16,
    side: u16,
    tiles: Vec<(u8, u32)>, // (type 0..=5, cells_mask)
    stab: &'static str,    // "STRICT" / "MARGINAL" / "UNSTABLE" / "showcase"
}

fn classify_for_manifest(cells: &[(u8, u8, u8)], hollow: bool) -> u8 {
    let n = cells.len();
    match (n, hollow) {
        (1, false) => 3,
        (1, true) => 5,
        (2, false) => 2,
        (2, true) => 4,
        (3, false) => {
            let xs: std::collections::HashSet<u8> = cells.iter().map(|c| c.0).collect();
            let ys: std::collections::HashSet<u8> = cells.iter().map(|c| c.1).collect();
            let zs: std::collections::HashSet<u8> = cells.iter().map(|c| c.2).collect();
            let varying = (xs.len() > 1) as u8 + (ys.len() > 1) as u8 + (zs.len() > 1) as u8;
            if varying == 1 { 0 } else { 1 }
        }
        _ => 99,
    }
}

fn cells_mask_u32_local(cells: &[(u8, u8, u8)]) -> u32 {
    cells.iter().fold(0u32, |a, &(x, y, z)| {
        a | (1u32 << (x as u32 * 9 + 3 * y as u32 + z as u32))
    })
}

fn add_to_manifest(name: &str, items: &[(Placement, bool)], label: &str) {
    let mut tiles: Vec<(u8, u32)> = Vec::new();
    let mut front: u16 = 0;
    let mut side: u16 = 0;
    for (p, hollow) in items {
        let kind = classify_for_manifest(&p.cells, *hollow);
        let cells = cells_mask_u32_local(&p.cells);
        tiles.push((kind, cells));
        if !*hollow {
            front |= p.front_mask;
            side |= p.side_mask;
        }
    }
    // Compute stability label. Cells outside the 3×3×3 grid are "showcase"
    // (e.g., the pieces overview); the LP isn't meaningful there.
    let placement_refs: Vec<&Placement> = items.iter().map(|(p, _)| p).collect();
    let cells_in_grid = items.iter().all(|(p, _)| {
        p.cells.iter().all(|&(x, y, z)| x <= 2 && y <= 2 && z <= 2)
    });
    let stab: &'static str = if !cells_in_grid {
        "showcase"
    } else if check_stability_quick(&placement_refs, 1e-4).is_stable() {
        "STRICT"
    } else if check_stability_quick(&placement_refs, 0.0).is_stable() {
        "MARGINAL"
    } else {
        "UNSTABLE"
    };
    let entry = ManifestEntry {
        label: label.to_string(),
        front, side, tiles, stab,
    };
    MANIFEST.with(|m| m.borrow_mut().push((name.to_string(), entry)));
}

fn write_manifest() {
    use std::fmt::Write as _;
    let mut json = String::from("{");
    MANIFEST.with(|m| {
        let entries = m.borrow();
        for (i, (name, e)) in entries.iter().enumerate() {
            if i > 0 { json.push(','); }
            write!(json, "\"{}\":{{\"label\":\"{}\",\"stab\":\"{}\",\"front\":{},\"side\":{},\"tiles\":[",
                   name, e.label.replace('"', "\\\""), e.stab, e.front, e.side).unwrap();
            for (j, (kind, cells)) in e.tiles.iter().enumerate() {
                if j > 0 { json.push(','); }
                write!(json, "[{},{}]", kind, cells).unwrap();
            }
            json.push_str("]}");
        }
    });
    json.push('}');
    // Synchronous-script form, loaded by tower_view.html via a regular
    // <script src> tag. (No fetch — works even when opened via file:// or
    // through a slow/flaky server.)
    let js = format!("window.EXAMPLES_MANIFEST = {};", json);
    std::fs::write(format!("{}/examples_manifest.js", ASSETS_DIR), js)
        .expect("write examples_manifest.js");
}

fn save_marked(items: Vec<(Placement, bool)>, name: &str, title: &str) {
    // Register in the manifest with the basename minus ".html".
    let stem = name.strip_suffix(".html").unwrap_or(name);
    let stem = stem.strip_prefix("tower_").unwrap_or(stem);
    add_to_manifest(stem, &items, title);
    // (Static HTML output suppressed — the blog uses tower_view.html with
    // the manifest. Keeping the call site signature for minimal diffs.)
}

// ---------------------------- pieces overview -------------------------------

/// Custom pieces-overview HTML. Lays out the six piece types in a 2x3 grid
/// (3 columns × 2 rows) so the iframe doesn't have to span a wide horizontal
/// slot. Hand-written Three.js scene; no stability LP, no reference cards
/// (this is just a "what's in the box" reference).
fn render_pieces_overview() {
    let path = format!("{}/pieces_overview.html", ASSETS_DIR);
    // Each piece is described as (cells, color, is_hollow, label).
    // Cells use small local coordinates that we then offset by slot.
    let pieces: Vec<(Vec<(i32, i32, i32)>, &str, bool, &str)> = vec![
        (vec![(0, 0, 0), (0, 0, 1), (0, 0, 2)], "#d62728", false, "triple"),
        (vec![(0, 0, 0), (1, 0, 0), (0, 1, 0)], "#2ca02c", false, "corner"),
        (vec![(0, 0, 0), (0, 0, 1)],            "#1f77b4", false, "double"),
        (vec![(0, 0, 0)],                        "#ff7f0e", false, "single"),
        (vec![(0, 0, 0), (0, 0, 1)],            "#9467bd", true,  "hollow double"),
        (vec![(0, 0, 0)],                        "#17becf", true,  "hollow single"),
    ];

    let cols: i32 = 3;
    // Tight spacing: corner piece is 2×2 in plan, so slots ≥ 2; with 2.6
     // there's a half-cell gap between adjacent pieces. Easier to read at
     // the iframe's modest size than the previous loose 4.0/4.5 layout.
    let slot_x: f64 = 2.6;
    let slot_z: f64 = 2.6;

    // Build JS data: each piece gets per-cell positions in three.js coords
    // (x, y_up, z_depth) directly — no Placement detour, so no bitmask issues.
    let mut data_js = String::from("[");
    for (i, (cells, color, hollow, label)) in pieces.iter().enumerate() {
        if i > 0 { data_js.push(','); }
        let col = (i as i32) % cols;
        let row = (i as i32) / cols;
        let ox = (col as f64) * slot_x;
        let oz = (row as f64) * slot_z;
        write!(data_js,
            "{{label:\"{}\",color:\"{}\",hollow:{},cells:[",
            label, color, hollow).unwrap();
        for (j, (cx, cy, cz)) in cells.iter().enumerate() {
            if j > 0 { data_js.push(','); }
            // (cx, cy, cz) are world (x, y, z); three.js maps to (x, z, y).
            let wx = ox + (*cx as f64);
            let wy = *cz as f64;
            let wz = oz + (*cy as f64);
            write!(data_js, "[{:.2},{:.2},{:.2}]", wx, wy, wz).unwrap();
        }
        data_js.push_str("]}");
    }
    data_js.push(']');

    // Camera target: centre of the grid.
    let target_x = ((cols - 1) as f64) * slot_x / 2.0;
    let target_z = slot_z / 2.0;

    let html = format!(
        r##"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8">
<title>pieces overview</title>
<style>html,body{{margin:0;height:100%;overflow:hidden;background:#f0f0f0;
font-family:sans-serif}}#legend{{position:absolute;top:10px;left:14px;
background:rgba(255,255,255,0.92);padding:6px 10px;border-radius:5px;
font-size:11px;color:#444;border:1px solid #bbb;line-height:1.5}}
#legend strong{{font-weight:600}}.sw{{display:inline-block;width:10px;
height:10px;border:1px solid #333;vertical-align:middle;margin:0 4px 0 0}}</style>
</head><body>
<div id="legend">
  <strong>The 6 piece types</strong> (one set; the box has two — red and blue)<br>
  <span class="sw" style="background:#d62728"></span>triple
  <span class="sw" style="background:#2ca02c;margin-left:8px"></span>corner
  <span class="sw" style="background:#1f77b4;margin-left:8px"></span>double
  <span class="sw" style="background:#ff7f0e;margin-left:8px"></span>single<br>
  <span class="sw" style="background:#9467bd"></span>hollow double
  <span class="sw" style="background:#17becf;margin-left:8px"></span>hollow single
</div>
<script type="importmap">
{{"imports":{{"three":"https://unpkg.com/three@0.160.0/build/three.module.js","three/addons/":"https://unpkg.com/three@0.160.0/examples/jsm/"}}}}
</script>
<script type="module">
import * as THREE from 'three';
import {{ OrbitControls }} from 'three/addons/controls/OrbitControls.js';
const PIECES = {data};
const scene = new THREE.Scene();
scene.background = new THREE.Color(0xf0f0f0);
const camera = new THREE.PerspectiveCamera(38, window.innerWidth/window.innerHeight, 0.1, 100);
camera.position.set({tx} + 5.5, 4.2, {tz} + 6.5);
const renderer = new THREE.WebGLRenderer({{ antialias: true }});
renderer.setSize(window.innerWidth, window.innerHeight);
renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5));
document.body.appendChild(renderer.domElement);
const controls = new OrbitControls(camera, renderer.domElement);
controls.target.set({tx}, 0.5, {tz});
controls.enableDamping = false;
controls.update();
scene.add(new THREE.AmbientLight(0xffffff, 0.6));
const sun = new THREE.DirectionalLight(0xffffff, 0.85);
sun.position.set(7, 10, 8);
scene.add(sun);
const fill = new THREE.DirectionalLight(0xffffff, 0.25);
fill.position.set(-5, 4, -3);
scene.add(fill);
// Ground.
const ground = new THREE.Mesh(
  new THREE.PlaneGeometry(20, 14),
  new THREE.MeshLambertMaterial({{ color: 0xdddddd, side: THREE.DoubleSide }})
);
ground.rotation.x = -Math.PI/2;
ground.position.set({tx}, 0, {tz});
scene.add(ground);
// Solid + hollow cell renderers.
const SCALE = 0.96;
const HALF = SCALE/2;
const HOLE_R = 0.36;
const cubeGeo = new THREE.BoxGeometry(SCALE, SCALE, SCALE);
const edgeGeo = new THREE.EdgesGeometry(cubeGeo);
const edgeMat = new THREE.LineBasicMaterial({{ color: 0x222222 }});
const panelShape = new THREE.Shape();
panelShape.moveTo(-HALF, -HALF); panelShape.lineTo( HALF, -HALF);
panelShape.lineTo( HALF,  HALF); panelShape.lineTo(-HALF,  HALF);
panelShape.lineTo(-HALF, -HALF);
const hole = new THREE.Path();
hole.absarc(0, 0, HOLE_R, 0, Math.PI*2, false);
panelShape.holes.push(hole);
const panelGeo = new THREE.ShapeGeometry(panelShape, 48);
function addSolid(x, y, z, mat) {{
  const c = new THREE.Mesh(cubeGeo, mat);
  c.position.set(x + 0.5, y + 0.5, z + 0.5);
  scene.add(c);
  const e = new THREE.LineSegments(edgeGeo, edgeMat);
  e.position.copy(c.position);
  scene.add(e);
}}
function addHollow(x, y, z, color) {{
  const mat = new THREE.MeshLambertMaterial({{
    color: color, side: THREE.DoubleSide,
    transparent: true, opacity: 0.55,
  }});
  const px = x + 0.5, py = y + 0.5, pz = z + 0.5;
  const faces = [
    [0, 0, 0, 0, HALF],
    [Math.PI, 0, 0, 0, -HALF],
    [0, Math.PI/2, HALF, 0, 0],
    [0, -Math.PI/2, -HALF, 0, 0],
    [-Math.PI/2, 0, 0, HALF, 0],
    [Math.PI/2, 0, 0, -HALF, 0],
  ];
  for (const [rx, ry, ox, oy, oz] of faces) {{
    const p = new THREE.Mesh(panelGeo, mat);
    p.rotation.x = rx; p.rotation.y = ry;
    p.position.set(px + ox, py + oy, pz + oz);
    scene.add(p);
  }}
}}
for (const piece of PIECES) {{
  const mat = new THREE.MeshLambertMaterial({{ color: piece.color }});
  for (const [x, y, z] of piece.cells) {{
    if (piece.hollow) addHollow(x, y, z, piece.color);
    else              addSolid(x, y, z, mat);
  }}
}}
function render() {{ renderer.render(scene, camera); }}
controls.addEventListener('change', render);
window.addEventListener('resize', () => {{
  camera.aspect = window.innerWidth/window.innerHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(window.innerWidth, window.innerHeight);
  render();
}});
render();
</script>
</body></html>
"##,
        data = data_js,
        tx = target_x,
        tz = target_z,
    );
    std::fs::write(&path, html).expect("write pieces_overview.html");
}

// --------------------------- balance demos ---------------------------------

fn render_l_on_hollow_stable() {
    // Corner cell of the L sits directly above the hollow base; the two arms
    // extend toward the (+x, +y) corner of the cube. Mirror of the previous
    // version, which extended toward (-x, -y).
    let items = vec![
        (Placement::new(vec![(1, 1, 0)], HOLLOW_CELL_MASS), true),
        (Placement::new(vec![(1, 1, 1), (2, 1, 1), (1, 2, 1)],
                        3.0 * FULL_CELL_MASS), false),
    ];
    save_marked(items, "tower_l_on_hollow_stable.html",
                "L-tromino on a hollow cube (stable)");
}

// (toppling L-on-hollow variant removed — Section 3 walks through stable
// configurations only, with the LP intuition introduced after them.)

/// Same L-on-hollow as the stable example, but with a 1×1×3 cantilevered on
/// top. Each piece's COM is over its own support, but the *cumulative* COM
/// of the L + 1×1×3 sits outside the hollow base's ground footprint, so the
/// whole tower tips. The point is that stability is not a per-block property
/// — you have to consider every block plus everything resting on it.
fn render_l_on_hollow_plus_triple() {
    let items = vec![
        // Hollow single base.
        (Placement::new(vec![(1, 1, 0)], HOLLOW_CELL_MASS), true),
        // L-tromino: corner over the base, arms toward (+x, +y) — mirror of
        // the (-x, -y) version that earlier examples used.
        (Placement::new(vec![(1, 1, 1), (2, 1, 1), (1, 2, 1)],
                        3.0 * FULL_CELL_MASS), false),
        // 1×1×3 on top at y = 2, mirroring the L's orientation. Its weight
        // pulls the cumulative COM past the +y edge of the base footprint.
        (Placement::new(vec![(0, 2, 2), (1, 2, 2), (2, 2, 2)],
                        3.0 * FULL_CELL_MASS), false),
    ];
    save_marked(items, "tower_l_on_hollow_plus_triple.html",
                "L on a hollow cube + 1×1×3 on top — tips cumulatively");
}

fn render_bridge() {
    let items = vec![
        (Placement::new(vec![(0, 0, 0)], FULL_CELL_MASS), false),
        (Placement::new(vec![(2, 0, 0)], FULL_CELL_MASS), false),
        (Placement::new(vec![(0, 0, 1), (1, 0, 1), (2, 0, 1)],
                        3.0 * FULL_CELL_MASS), false),
    ];
    save_marked(items, "tower_bridge.html",
                "bridge: two pillars + 1×1×3 span");
}

fn render_l_hugging() {
    // Vertical 1×1×2 with an L-tromino wrapping back-and-down behind it.
    // The L's only support is the contact at (0,0,2)/(0,0,1); its COM at
    // y = 2/3 sits outside the 1×1 support polygon, so this configuration
    // tips. Used in Section 3 as the "looks like it should work but doesn't"
    // case that motivates the LP-based check.
    let items = vec![
        (Placement::new(vec![(0, 0, 0), (0, 0, 1)],
                        2.0 * FULL_CELL_MASS), false),
        (Placement::new(vec![(0, 0, 2), (0, 1, 2), (0, 1, 1)],
                        3.0 * FULL_CELL_MASS), false),
    ];
    save_marked(items, "tower_l_hugging.html",
                "L hugging a vertical 1×1×2 — tips");
}

fn render_cantilever_ok() {
    let items = vec![
        (Placement::new(vec![(0, 0, 0), (1, 0, 0), (2, 0, 0)],
                        3.0 * FULL_CELL_MASS), false),
        (Placement::new(vec![(0, 0, 1)], FULL_CELL_MASS), false),
    ];
    save_marked(items, "tower_cantilever_ok.html",
                "cantilever: 1×1×3 + single on the supported end (stable)");
}

fn render_weight_actual() {
    // Configuration the user specified, rendered at the actual game mass
    // ratio (full = 150 g/cell, hollow = 50 g/cell, ratio 3:1).
    let items = vec![
        (Placement::new(vec![(0, 0, 0), (0, 0, 1)], 2.0 * FULL_CELL_MASS), false),
        (Placement::new(vec![(0, 1, 0)], FULL_CELL_MASS), false),
        (Placement::new(vec![(0, 1, 1), (0, 2, 1), (0, 2, 2)], 3.0 * FULL_CELL_MASS), false),
        (Placement::new(vec![(0, 0, 2), (0, 1, 2)], 2.0 * HOLLOW_CELL_MASS), true),
    ];
    save_marked(items, "tower_weight_actual.html",
                "weight matters — game ratio (full 150 g, hollow 50 g)");
}

fn render_weight_extreme() {
    // Same geometry as weight_actual but with hollow blocks made nearly
    // weightless. The reduced counterweight no longer holds the cantilever:
    // the LP that succeeded at 3:1 fails at 15:1.
    let items = vec![
        (Placement::new(vec![(0, 0, 0), (0, 0, 1)], 2.0 * 150.0), false),
        (Placement::new(vec![(0, 1, 0)], 150.0), false),
        (Placement::new(vec![(0, 1, 1), (0, 2, 1), (0, 2, 2)], 3.0 * 150.0), false),
        (Placement::new(vec![(0, 0, 2), (0, 1, 2)], 2.0 * 10.0), true),
    ];
    save_marked(items, "tower_weight_extreme.html",
                "same geometry — hollow nearly weightless (15:1)");
}

// --------------------------- silhouette helpers ----------------------------

/// Parse "row|row|row" (top row first) into a 9-bit mask with bit `(a*3 + z)`
/// set iff column `a`, row `z` is filled (z=0 = bottom). Same convention as
/// `view_counts.txt` / `render_view` in main.rs.
fn parse_grid(s: &str) -> u16 {
    let rows: Vec<&str> = s.split('|').collect();
    assert_eq!(rows.len(), 3, "expected 3 rows in {:?}", s);
    let mut m = 0u16;
    for (i, row) in rows.iter().enumerate() {
        let z = 2 - i;
        for (a, c) in row.chars().enumerate() {
            if c == '#' {
                m |= 1u16 << (a * 3 + z);
            }
        }
    }
    m
}

fn render_grid(mask: u16) -> String {
    let mut s = String::with_capacity(11);
    for z in (0..3).rev() {
        for a in 0..3 {
            s.push(if (mask >> (a * 3 + z)) & 1 == 1 { '#' } else { '.' });
        }
        if z > 0 {
            s.push('|');
        }
    }
    s
}

fn rotate90(mask: u16) -> u16 {
    let mut n = 0u16;
    for a in 0..3u16 {
        for z in 0..3u16 {
            if (mask >> (a * 3 + z)) & 1 == 1 {
                n |= 1u16 << (z * 3 + (2 - a));
            }
        }
    }
    n
}

fn flip_h(mask: u16) -> u16 {
    let mut n = 0u16;
    for a in 0..3u16 {
        for z in 0..3u16 {
            if (mask >> (a * 3 + z)) & 1 == 1 {
                n |= 1u16 << ((2 - a) * 3 + z);
            }
        }
    }
    n
}

fn d4_orbit(mask: u16) -> Vec<u16> {
    let mut s = std::collections::HashSet::new();
    let mut c = mask;
    for _ in 0..4 {
        s.insert(c);
        s.insert(flip_h(c));
        c = rotate90(c);
    }
    let mut v: Vec<u16> = s.into_iter().collect();
    v.sort();
    v
}

// --------------------------- view_counts loader ----------------------------

fn read_views(path: &str) -> Vec<(u16, u16, u64, u64)> {
    let f = File::open(path).unwrap_or_else(|e| panic!("open {}: {}", path, e));
    let mut out = Vec::new();
    for line in BufReader::new(f).lines() {
        let line = line.unwrap();
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') {
            continue;
        }
        let mut p = s.split_whitespace();
        let strict: u64 = match p.next().and_then(|x| x.parse().ok()) {
            Some(v) => v,
            None => continue,
        };
        let marginal: u64 = match p.next().and_then(|x| x.parse().ok()) {
            Some(v) => v,
            None => continue,
        };
        let front = parse_grid(p.next().unwrap());
        let side = parse_grid(p.next().unwrap());
        out.push((front, side, strict, marginal));
    }
    out
}

fn read_game_cards() -> Vec<u16> {
    let f = File::open("cards_original.txt").expect("cards_original.txt");
    BufReader::new(f)
        .lines()
        .filter_map(|l| l.ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .filter_map(|s| s.split_whitespace().nth(1).map(|p| parse_grid(p)))
        .collect()
}

// --------------------------- tower search ---------------------------------

/// Find a stable arrangement of pieces whose front silhouette equals
/// `target_front` and side silhouette equals `target_side`. Returns
/// `(placement, is_hollow)` pairs in ready-to-render order, or `None`.
///
/// Prefers a strictly-stable arrangement (margin = 1e-4); falls back to a
/// marginally-stable one if none is strict. That keeps the rendered tower's
/// label aligned with the post's prose: when we say "this pair has 3 stable
/// solutions" we want the rendered example to show STRICT, not MARGINAL.
fn find_tower(target_front: u16, target_side: u16) -> Option<Vec<(Placement, bool)>> {
    if let Some(t) = find_tower_at_margin(target_front, target_side, 1e-4) {
        return Some(t);
    }
    find_tower_at_margin(target_front, target_side, 0.0)
}

fn find_tower_at_margin(
    target_front: u16,
    target_side: u16,
    margin: f64,
) -> Option<Vec<(Placement, bool)>> {
    // Same placement universe as main.rs.
    let single_shapes: Vec<Vec<(i32, i32, i32)>> = vec![vec![(0, 0, 0)]];
    let double_shapes: Vec<Vec<(i32, i32, i32)>> = vec![
        vec![(0, 0, 0), (1, 0, 0)],
        vec![(0, 0, 0), (0, 1, 0)],
        vec![(0, 0, 0), (0, 0, 1)],
    ];
    let triple_shapes: Vec<Vec<(i32, i32, i32)>> = vec![
        vec![(0, 0, 0), (1, 0, 0), (2, 0, 0)],
        vec![(0, 0, 0), (0, 1, 0), (0, 2, 0)],
        vec![(0, 0, 0), (0, 0, 1), (0, 0, 2)],
    ];
    let mut corner_shapes: Vec<Vec<(i32, i32, i32)>> = Vec::new();
    for &(da, db) in &[
        ((1, 0), (0, 1)), ((1, 0), (0, -1)),
        ((-1, 0), (0, 1)), ((-1, 0), (0, -1)),
    ] {
        corner_shapes.push(vec![(0,0,0),(da.0,da.1,0),(da.0+db.0,da.1+db.1,0)]);
        corner_shapes.push(vec![(0,0,0),(da.0,0,da.1),(da.0+db.0,0,da.1+db.1)]);
        corner_shapes.push(vec![(0,0,0),(0,da.0,da.1),(0,da.0+db.0,da.1+db.1)]);
    }

    fn enumerate(shapes: &[Vec<(i32, i32, i32)>], mass: f64) -> Vec<Placement> {
        let mut by_mask: HashMap<u64, Vec<(u8, u8, u8)>> = HashMap::new();
        for x in 0..=2i32 {
            for y in 0..=2i32 {
                for z in 0..=2i32 {
                    'shape: for shape in shapes {
                        let mut v = Vec::with_capacity(shape.len());
                        for c in shape {
                            let (tx, ty, tz) = (c.0 + x, c.1 + y, c.2 + z);
                            if !(0..=2).contains(&tx)
                                || !(0..=2).contains(&ty)
                                || !(0..=2).contains(&tz)
                            {
                                continue 'shape;
                            }
                            v.push((tx as u8, ty as u8, tz as u8));
                        }
                        let mask = v.iter().fold(0u64, |a, &(x, y, z)| {
                            a | (1u64 << (x as u32 * 9 + 3 * y as u32 + z as u32))
                        });
                        by_mask.entry(mask).or_insert(v);
                    }
                }
            }
        }
        let mut v: Vec<Placement> = by_mask
            .into_iter()
            .map(|(_, c)| Placement::new(c, mass))
            .collect();
        v.sort_by_key(|p| p.occupy);
        v
    }

    let p_triple = enumerate(&triple_shapes, 3.0 * FULL_CELL_MASS);
    let p_corner = enumerate(&corner_shapes, 3.0 * FULL_CELL_MASS);
    let p_double = enumerate(&double_shapes, 2.0 * FULL_CELL_MASS);
    let p_single = enumerate(&single_shapes, FULL_CELL_MASS);
    let p_hdouble = enumerate(&double_shapes, 2.0 * HOLLOW_CELL_MASS);
    let p_hsingle = enumerate(&single_shapes, HOLLOW_CELL_MASS);

    // Brute-force search like main.rs's loop, returning the first stable hit.
    // Allows skipping any of the four visible tiles too — this means a target
    // front/side silhouette of (e.g.) just the triple alone is fair game.
    let triples: Vec<Option<&Placement>> =
        p_triple.iter().map(Some).chain(std::iter::once(None)).collect();
    let corners: Vec<Option<&Placement>> =
        p_corner.iter().map(Some).chain(std::iter::once(None)).collect();
    let doubles: Vec<Option<&Placement>> =
        p_double.iter().map(Some).chain(std::iter::once(None)).collect();
    let singles: Vec<Option<&Placement>> =
        p_single.iter().map(Some).chain(std::iter::once(None)).collect();
    let hdoubles: Vec<Option<&Placement>> =
        p_hdouble.iter().map(Some).chain(std::iter::once(None)).collect();
    let hsingles: Vec<Option<&Placement>> =
        p_hsingle.iter().map(Some).chain(std::iter::once(None)).collect();

    let unpack = |p: Option<&Placement>| -> (u64, u16, u16) {
        match p {
            Some(p) => (p.occupy, p.front_mask, p.side_mask),
            None => (0u64, 0u16, 0u16),
        }
    };

    for p1 in &triples {
        let (o1, f1, s1) = unpack(*p1);
        for p2 in &corners {
            if let Some(p) = *p2 {
                if o1 & p.occupy != 0 { continue; }
            }
            let (do2, df2, ds2) = unpack(*p2);
            let o2 = o1 | do2; let f2 = f1 | df2; let s2 = s1 | ds2;
            for p3 in &doubles {
                if let Some(p) = *p3 {
                    if o2 & p.occupy != 0 { continue; }
                }
                let (do3, df3, ds3) = unpack(*p3);
                let o3 = o2 | do3; let f3 = f2 | df3; let s3 = s2 | ds3;
                for p4 in &singles {
                    if let Some(p) = *p4 {
                        if o3 & p.occupy != 0 { continue; }
                    }
                    let (do4, df4, ds4) = unpack(*p4);
                    let o4 = o3 | do4;
                    let front = f3 | df4;
                    let side = s3 | ds4;
                    if front != target_front || side != target_side { continue; }

                    for p5 in &hdoubles {
                        if let Some(p) = *p5 {
                            if o4 & p.occupy != 0 { continue; }
                        }
                        let do5 = p5.map_or(0u64, |p| p.occupy);
                        let o5 = o4 | do5;
                        for p6 in &hsingles {
                            if let Some(p) = *p6 {
                                if o5 & p.occupy != 0 { continue; }
                            }
                            let mut active: Vec<&Placement> = Vec::new();
                            if let Some(p) = *p1 { active.push(p); }
                            if let Some(p) = *p2 { active.push(p); }
                            if let Some(p) = *p3 { active.push(p); }
                            if let Some(p) = *p4 { active.push(p); }
                            if let Some(p) = *p5 { active.push(p); }
                            if let Some(p) = *p6 { active.push(p); }
                            if active.is_empty() { continue; }
                            if check_stability_quick(&active, margin).is_stable() {
                                let mut out: Vec<(Placement, bool)> = Vec::new();
                                if let Some(p) = *p1 { out.push((p.clone(), false)); }
                                if let Some(p) = *p2 { out.push((p.clone(), false)); }
                                if let Some(p) = *p3 { out.push((p.clone(), false)); }
                                if let Some(p) = *p4 { out.push((p.clone(), false)); }
                                if let Some(p) = *p5 { out.push((p.clone(), true)); }
                                if let Some(p) = *p6 { out.push((p.clone(), true)); }
                                return Some(out);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

// --------------------------- compat graph copy ------------------------------

/// Run graph_viz twice: once with the default view_counts.txt (subset-allowed
/// regime), once with view_counts_all.txt (all-pieces-required regime). Each
/// run writes its own compat_graph file in blog/assets/.
fn copy_compat_graph() {
    let runs = [
        ("view_counts.txt", "compat_graph_subset.html"),
        ("view_counts_all.txt", "compat_graph_all.html"),
    ];
    for (input, output) in &runs {
        let target_path = format!("{}/{}", ASSETS_DIR, output);
        let status = std::process::Command::new("./target/release/graph_viz")
            .env("PROJECT_CUBE_VIEW_COUNTS", input)
            .env("PROJECT_CUBE_GRAPH_OUTPUT", &target_path)
            .status();
        if !matches!(status, Ok(s) if s.success()) {
            eprintln!("warning: graph_viz failed for {}", input);
            continue;
        }
        eprintln!("wrote {}", target_path);
    }
}

// --------------------------- card SVGs --------------------------------------

fn card_svg(mask: u16, size_px: u32) -> String {
    let cell = (size_px - 4) / 3;
    let mut svg = format!(
        r#"<svg xmlns='http://www.w3.org/2000/svg' width='{s}' height='{s}' viewBox='0 0 {s} {s}'>"#,
        s = size_px
    );
    svg.push_str(
        r#"<rect width='100%' height='100%' fill='white' stroke='#222' stroke-width='1.5'/>"#,
    );
    for z in 0..3u32 {
        for a in 0..3u32 {
            if (mask >> (a * 3 + z)) & 1 == 1 {
                let x = 2 + a * cell;
                let y = 2 + (2 - z) * cell;
                svg.push_str(&format!(
                    "<rect x='{x}' y='{y}' width='{cell}' height='{cell}' fill='#333'/>",
                    x = x, y = y, cell = cell
                ));
            }
        }
    }
    svg.push_str("</svg>");
    svg
}

fn render_cards_svg() {
    let cards: Vec<(u32, String)> = BufReader::new(File::open("cards_original.txt").unwrap())
        .lines()
        .filter_map(|l| l.ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .map(|s| {
            let mut p = s.split_whitespace();
            let id: u32 = p.next().unwrap().parse().unwrap();
            let pat = p.next().unwrap().to_string();
            (id, pat)
        })
        .collect();

    let mut html = String::from(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>Project Cube cards</title>
<style>html,body{margin:0;padding:0}body{font-family:sans-serif;padding:2px 14px 6px;background:#fafaf7}.grid{display:grid;grid-template-columns:repeat(5,1fr);gap:8px;max-width:640px}.card{text-align:center;background:#fff;padding:4px;border-radius:5px;border:1px solid #e2e2e2}.card .num{font-size:10px;color:#666;margin-top:2px}h2{margin:2px 0 6px 0;font-family:Georgia,serif;font-size:1em}</style>
</head><body><h2>Project Cube — the 15 cards</h2><div class="grid">"#,
    );
    for (id, pat) in &cards {
        let mask = parse_grid(pat);
        let svg = card_svg(mask, 70);
        html.push_str(&format!(
            r#"<div class="card">{svg}<div class="num">card {id}</div></div>"#,
            svg = svg, id = id
        ));
    }
    html.push_str("</div></body></html>");
    std::fs::write(format!("{}/cards.html", ASSETS_DIR), html).unwrap();
}

// --------------------------- data-driven renders ----------------------------

fn render_hardest_pair() {
    let mut views = read_views("view_counts.txt");
    views.retain(|(_, _, s, _)| *s > 0);
    views.sort_by_key(|(_, _, s, _)| *s);
    let (front, side, strict, _) = views[0];
    eprintln!(
        "hardest pair: STRICT = {} | front = {} | side = {}",
        strict, render_grid(front), render_grid(side)
    );
    let tower = find_tower(front, side).expect("hardest tower must exist");
    save_marked(
        tower,
        "tower_hardest_pair.html",
        &format!(
            "hardest pair (1 stable solution) — front {} / side {}",
            render_grid(front),
            render_grid(side)
        ),
    );
}

fn render_skip_required() -> Option<(u32, &'static str, u16, u32, &'static str, u16, u64)> {
    // Only consider rotations of distinct printed-deck cards (the user's
    // request: this example should use the actual game cards).
    let cards_with_ids: Vec<(u32, u16)> = BufReader::new(
        File::open("cards_original.txt").expect("cards_original.txt"))
        .lines().filter_map(|l| l.ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .filter_map(|s| {
            let mut p = s.split_whitespace();
            let id: u32 = p.next()?.parse().ok()?;
            Some((id, parse_grid(p.next()?)))
        })
        .collect();
    let rot_labels: [&'static str; 4] = ["0°", "90°", "180°", "270°"];
    let mut card_rotations: Vec<(u32, usize, u16)> = Vec::new();
    for &(id, mask) in &cards_with_ids {
        let mut m = mask;
        for r in 0..4 {
            card_rotations.push((id, r, m));
            m = rotate90(m);
        }
    }

    let def: HashMap<(u16, u16), (u64, u64)> =
        read_views("view_counts.txt").into_iter()
            .map(|(f, s, st, mg)| ((f, s), (st, mg))).collect();
    let strict_set: std::collections::HashSet<(u16, u16)> =
        read_views("view_counts_all.txt").into_iter()
            .filter(|(_, _, st, _)| *st > 0)
            .map(|(f, s, _, _)| (f, s))
            .collect();

    let mut found: Option<(u32, usize, u32, usize, u16, u16, u64)> = None;
    'outer: for &(id_f, rot_f, m_f) in &card_rotations {
        for &(id_s, rot_s, m_s) in &card_rotations {
            if id_f == id_s { continue; }
            if let Some(&(st, _)) = def.get(&(m_f, m_s)) {
                if st > 0 && !strict_set.contains(&(m_f, m_s)) {
                    found = Some((id_f, rot_f, id_s, rot_s, m_f, m_s, st));
                    break 'outer;
                }
            }
        }
    }

    let (id_f, rot_f, id_s, rot_s, m_f, m_s, st) =
        found.expect("a game-card skip-required pair must exist");
    eprintln!(
        "skip-required game pair: card {} ({}) × card {} ({}), STRICT = {}",
        id_f, rot_labels[rot_f], id_s, rot_labels[rot_s], st
    );
    let tower = find_tower(m_f, m_s).expect("skip-required tower must be findable");
    save_marked(
        tower,
        "tower_skip_required.html",
        &format!(
            "card {} ({}) × card {} ({}) — only solvable by skipping a piece",
            id_f, rot_labels[rot_f], id_s, rot_labels[rot_s]
        ),
    );
    Some((id_f, rot_labels[rot_f], m_f, id_s, rot_labels[rot_s], m_s, st))
}

/// Easiest game-card pair: among (orientation_of_card_i, orientation_of_card_j)
/// for distinct printed-deck cards, find the pair with the largest STRICT
/// count and render one of its solutions. Writes a manifest line that the
/// post uses to label the figure.
fn render_easiest_game_pair() -> Option<(u32, &'static str, u16, u32, &'static str, u16, u64)> {
    let cards_with_ids: Vec<(u32, u16)> = BufReader::new(
        File::open("cards_original.txt").expect("cards_original.txt"))
        .lines().filter_map(|l| l.ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .filter_map(|s| {
            let mut p = s.split_whitespace();
            let id: u32 = p.next()?.parse().ok()?;
            let mask = parse_grid(p.next()?);
            Some((id, mask))
        })
        .collect();
    let rot_labels: [&'static str; 4] = ["0°", "90°", "180°", "270°"];

    let mut card_rotations: Vec<(u32, usize, u16)> = Vec::new();
    for &(id, mask) in &cards_with_ids {
        let mut m = mask;
        for r in 0..4 {
            card_rotations.push((id, r, m));
            m = rotate90(m);
        }
    }
    let views: HashMap<(u16, u16), (u64, u64)> =
        read_views("view_counts.txt").into_iter()
            .map(|(f, s, st, mg)| ((f, s), (st, mg))).collect();

    let mut best: Option<(u32, usize, u32, usize, u16, u16, u64)> = None;
    for &(id_f, rot_f, m_f) in &card_rotations {
        for &(id_s, rot_s, m_s) in &card_rotations {
            if id_f == id_s { continue; } // distinct printed cards
            if let Some(&(st, _)) = views.get(&(m_f, m_s)) {
                if best.map_or(true, |b| st > b.6) {
                    best = Some((id_f, rot_f, id_s, rot_s, m_f, m_s, st));
                }
            }
        }
    }

    let (id_f, rot_f, id_s, rot_s, m_f, m_s, st) = best?;
    let tower = find_tower(m_f, m_s).expect("must find easiest game tower");
    save_marked(
        tower,
        "tower_easiest_game_pair.html",
        &format!(
            "easiest game pair: card {} ({}) × card {} ({}) — {} stable",
            id_f, rot_labels[rot_f], id_s, rot_labels[rot_s], st
        ),
    );
    eprintln!(
        "easiest game pair: card {} ({}) × card {} ({}) — STRICT = {}",
        id_f, rot_labels[rot_f], id_s, rot_labels[rot_s], st
    );
    Some((id_f, rot_labels[rot_f], m_f, id_s, rot_labels[rot_s], m_s, st))
}

/// Hardest game-card pairs: pairs of (card, rotation) drawn from the printed
/// deck whose silhouettes admit very few stable solutions. Renders one tower
/// per pair plus a JSON manifest the blog post can use to populate a "try
/// to solve it yourself" widget.
fn render_hardest_pairs_solver() {
    // Read the printed deck with IDs.
    let cards_with_ids: Vec<(u32, u16)> = BufReader::new(
        File::open("cards_original.txt").expect("cards_original.txt"))
        .lines()
        .filter_map(|l| l.ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .filter_map(|s| {
            let mut p = s.split_whitespace();
            let id: u32 = p.next()?.parse().ok()?;
            let mask = parse_grid(p.next()?);
            Some((id, mask))
        })
        .collect();

    // Each (card_id, rotation_index, rotated_mask). Rotations only — the deck
    // doesn't allow reflections (the printed art has chirality). For each
    // card we deduplicate rotations under reflection: if two of the four 90°
    // rotations are mirrors of each other (i.e., the card's rot_180 equals
    // its own horizontal flip, or similar), keep only one of them. Listing
    // both would produce silhouettes that differ only by reflection, which
    // we already exclude at the pair level.
    let rot_labels = ["0°", "90°", "180°", "270°"];
    let mut card_rotations: Vec<(u32, usize, u16)> = Vec::new();
    for &(id, mask) in &cards_with_ids {
        let mut m = mask;
        let mut seen_ref_classes: std::collections::HashSet<u16> =
            std::collections::HashSet::new();
        for r in 0..4 {
            let class_key = std::cmp::min(m, flip_h(m));
            if seen_ref_classes.insert(class_key) {
                card_rotations.push((id, r, m));
            }
            m = rotate90(m);
        }
    }

    let views: HashMap<(u16, u16), (u64, u64)> =
        read_views("view_counts.txt").into_iter()
            .map(|(f, s, st, mg)| ((f, s), (st, mg)))
            .collect();

    // Score every (front, side) pair by STRICT count, only counting positive.
    // Tuple order: (id_f, rot_f, id_s, rot_s, m_f, m_s, strict, marginal).
    let mut pair_data: Vec<(u32, usize, u32, usize, u16, u16, u64, u64)> = Vec::new();
    for &(id_f, rot_f, m_f) in &card_rotations {
        for &(id_s, rot_s, m_s) in &card_rotations {
            if id_f == id_s { continue; } // pair must be two distinct cards
            if let Some(&(st, mg)) = views.get(&(m_f, m_s)) {
                if st == 1 {
                    pair_data.push((id_f, rot_f, id_s, rot_s, m_f, m_s, st, mg));
                }
            }
        }
    }

    pair_data.sort_by_key(|t| t.6);

    // Deduplicate by the silhouette pair's full equivalence class. Two scenarios
    // are the same tower up to:
    //   1. applying the same D4 element to both silhouettes (rotating or
    //      mirroring the whole holder), and/or
    //   2. swapping front ↔ side (rotating the tower 90° around the vertical
    //      axis — same physical structure, different observer).
    // Combine: take the lex-min over all 8 D4 transformations × 2 orientations.
    fn canonical_pair(f: u16, s: u16) -> (u16, u16) {
        let mut best = (f, s);
        let consider = |a: u16, b: u16, best: &mut (u16, u16)| {
            if (a, b) < *best { *best = (a, b); }
            if (b, a) < *best { *best = (b, a); }
        };
        let mut cf = f;
        let mut cs = s;
        for _ in 0..4 {
            consider(cf, cs, &mut best);
            consider(flip_h(cf), flip_h(cs), &mut best);
            cf = rotate90(cf);
            cs = rotate90(cs);
        }
        best
    }
    let mut chosen: Vec<(u32, usize, u32, usize, u16, u16, u64, u64)> = Vec::new();
    let mut seen: std::collections::HashSet<(u16, u16)> =
        std::collections::HashSet::new();
    for entry in pair_data {
        let key = canonical_pair(entry.4, entry.5);
        if seen.insert(key) {
            chosen.push(entry);
        }
    }

    // Sort: marginal asc (more marginal pairs sink to the bottom), then
    // front-card id, front-card rotation, side-card id.
    chosen.sort_by_key(|t| (t.7, t.0, t.1, t.2));

    // Render a tower for each pair; build manifest in parallel.
    let mut manifest = String::from("[");
    for (i, &(id_f, rot_f, id_s, rot_s, m_f, m_s, strict, marginal)) in chosen.iter().enumerate() {
        let tower = match find_tower(m_f, m_s) {
            Some(t) => t,
            None => continue, // shouldn't happen — view_counts said STRICT > 0
        };
        let filename = format!("hardpair_{:02}.html", i);
        save_marked(
            tower,
            &filename,
            &format!("hard pair: card {} ({}) × card {} ({})",
                     id_f, rot_labels[rot_f], id_s, rot_labels[rot_s]),
        );
        if i > 0 { manifest.push(','); }
        write!(
            manifest,
            r#"{{"front":{{"card":{cf},"rot":"{rf}","mask":{mf}}},"side":{{"card":{cs},"rot":"{rs}","mask":{ms}}},"strict":{st},"marginal":{mg},"file":"{fname}"}}"#,
            cf = id_f, rf = rot_labels[rot_f], mf = m_f,
            cs = id_s, rs = rot_labels[rot_s], ms = m_s,
            st = strict, mg = marginal, fname = filename,
        ).unwrap();
    }
    manifest.push(']');

    std::fs::write(format!("{}/hardpairs_manifest.js", ASSETS_DIR),
                   format!("window.HARDPAIRS_MANIFEST = {};", manifest))
        .expect("write hardpairs manifest (.js)");
    eprintln!("hardest-pairs solver: {} pairs", chosen.len());
}

fn render_extension_marginal(extra_pat: &str, name: &str, title_extra: &str) {
    let extra = parse_grid(extra_pat);
    let extra_orients = d4_orbit(extra);
    let mut counts: HashMap<(u16, u16), (u64, u64)> = HashMap::new();
    for (f, s, st, mg) in read_views("view_counts.txt") {
        counts.insert((f, s), (st, mg));
    }
    let game = read_game_cards();
    let mut found: Option<(u16, u16)> = None;
    'outer: for &gc in &game {
        for go in d4_orbit(gc) {
            for &eo in &extra_orients {
                for &(f, s) in &[(go, eo), (eo, go)] {
                    if let Some(&(st, mg)) = counts.get(&(f, s)) {
                        if st == 0 && mg > 0 {
                            found = Some((f, s));
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
    let (f, s) = found
        .unwrap_or_else(|| panic!("no marginal-only pair found for {}", extra_pat));
    eprintln!(
        "extension marginal ({}): front {} | side {}",
        title_extra, render_grid(f), render_grid(s)
    );
    let tower = find_tower(f, s).expect("marginal tower must exist");
    save_marked(
        tower,
        name,
        &format!("marginal-only pair when adding {} — front {} / side {}",
                 title_extra, render_grid(f), render_grid(s)),
    );
}

fn render_extension_impossible_both() {
    let extra1 = parse_grid(".#.|###|###");
    let extra2 = parse_grid("#..|###|###");
    let or1 = d4_orbit(extra1);
    let or2 = d4_orbit(extra2);
    let mut counts: HashMap<(u16, u16), (u64, u64)> = HashMap::new();
    for (f, s, st, mg) in read_views("view_counts.txt") {
        counts.insert((f, s), (st, mg));
    }
    let mut found: Option<(u16, u16)> = None;
    'outer: for &a in &or1 {
        for &b in &or2 {
            for &(f, s) in &[(a, b), (b, a)] {
                let bad = match counts.get(&(f, s)) {
                    None => true,
                    Some(&(st, mg)) => st == 0 && mg == 0,
                };
                if bad {
                    found = Some((f, s));
                    break 'outer;
                }
            }
        }
    }
    let (f, s) = found.expect("impossible pair must exist");
    eprintln!(
        "impossible pair (both added): front {} | side {}",
        render_grid(f), render_grid(s)
    );
    // Render two large silhouette SVGs side by side so the impossible pair
    // is visible at a glance.
    let svg_for_mask = |mask: u16| -> String {
        let size = 180u32;
        let cell = (size - 8) / 3;
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{s}" height="{s}" viewBox="0 0 {s} {s}">"#,
            s = size
        );
        svg.push_str(r##"<rect width="100%" height="100%" fill="#fffbf0" stroke="#5a4a30" stroke-width="3"/>"##);
        for z in 0..3u32 {
            for a in 0..3u32 {
                if (mask >> (a * 3 + z)) & 1 == 1 {
                    let x = 4 + a * cell;
                    let y = 4 + (2 - z) * cell;
                    svg.push_str(&format!(
                        r##"<rect x="{x}" y="{y}" width="{c}" height="{c}" fill="#3a3a3a"/>"##,
                        x = x, y = y, c = cell
                    ));
                }
            }
        }
        svg.push_str("</svg>");
        svg
    };

    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>impossible pair</title>
<style>html,body{{margin:0;height:100%;overflow:hidden}}body{{font-family:Georgia,serif;background:#fafaf7;color:#222;
padding:24px 16px;text-align:center;box-sizing:border-box}}h2{{margin:0 0 10px 0;font-size:1.3em}}
.lead{{max-width:520px;margin:0 auto 24px;font-size:0.95em;color:#555}}
.cards{{display:flex;justify-content:center;align-items:center;gap:28px;flex-wrap:wrap}}
.card-block{{display:flex;flex-direction:column;align-items:center;gap:6px}}
.card-block .label{{font-size:0.85em;color:#777;text-transform:uppercase;letter-spacing:0.05em}}
.cross{{font-size:2em;color:#aaa;font-weight:300}}</style>
</head><body>
<h2>Impossible pair</h2>
<p class="lead">No arrangement of pieces matches both silhouettes. This pair appears once you've added <em>both</em> <code>.#.|###|###</code> and <code>#..|###|###</code> to the deck — neither silhouette breaks anything on its own, but in combination they refer to a tower that cannot exist.</p>
<div class="cards">
  <div class="card-block">
    <div class="label">front</div>
    {front_svg}
  </div>
  <div class="cross">×</div>
  <div class="card-block">
    <div class="label">side</div>
    {side_svg}
  </div>
</div>
</body></html>"#,
        front_svg = svg_for_mask(f),
        side_svg = svg_for_mask(s),
    );
    std::fs::write(format!("{}/tower_extend_impossible_both.html", ASSETS_DIR), html)
        .expect("write impossible html");
}

fn main() {
    std::fs::create_dir_all(ASSETS_DIR).expect("create assets dir");
    eprintln!("Writing blog assets to {}/", ASSETS_DIR);

    render_pieces_overview();
    render_l_on_hollow_stable();
    render_l_on_hollow_plus_triple();
    render_bridge();
    render_l_hugging();
    render_cantilever_ok();
    render_weight_actual();
    render_weight_extreme();

    render_hardest_pair();
    let skip_pair = render_skip_required();
    let easiest_game = render_easiest_game_pair();
    render_hardest_pairs_solver();

    // stats.json — small per-section facts the blog HTML pulls in by JS so
    // the prose stays in sync with whatever the data currently says.
    let mut stats = String::from("{");
    if let Some((cf, rf, mf, cs, rs, ms, st)) = easiest_game {
        write!(stats,
            r#""easiest_game":{{"front_card":{cf},"front_rot":"{rf}","front_mask":{mf},"side_card":{cs},"side_rot":"{rs}","side_mask":{ms},"strict":{st}}}"#,
            cf=cf, rf=rf, mf=mf, cs=cs, rs=rs, ms=ms, st=st
        ).unwrap();
    }
    if let Some((cf, rf, mf, cs, rs, ms, st)) = skip_pair {
        if !stats.ends_with('{') { stats.push(','); }
        write!(stats,
            r#""skip_required":{{"front_card":{cf},"front_rot":"{rf}","front_mask":{mf},"side_card":{cs},"side_rot":"{rs}","side_mask":{ms},"strict":{st}}}"#,
            cf=cf, rf=rf, mf=mf, cs=cs, rs=rs, ms=ms, st=st
        ).unwrap();
    }
    stats.push('}');
    std::fs::write(format!("{}/stats.js", ASSETS_DIR),
                   format!("window.STATS = {};", stats))
        .expect("write stats.js");
    render_extension_marginal(
        ".#.|###|###",
        "tower_extend_marg_dotcenter.html",
        ".#.|###|###",
    );
    render_extension_marginal(
        "#..|###|###",
        "tower_extend_marg_corner.html",
        "#..|###|###",
    );
    render_extension_impossible_both();

    render_cards_svg();
    copy_compat_graph();

    write_manifest();
}
