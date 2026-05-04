//! Self-contained HTML+Three.js renderer for 6-tile cube configurations.
//!
//! Used by the enumeration loop in `main.rs` to capture configurations whose
//! stability flips when the empty tiles' masses change. Each output is a
//! standalone HTML file: open it in a browser, drag to rotate, scroll to
//! zoom. Three.js is pulled from a CDN at view-time; no build step needed.

use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::balance::{check_stability_quick, Placement};

/// Hard cap so a runaway hot loop can't fill the disk.
const MAX_IMAGES: usize = 100;

/// Margin used by the renderer's "STRICT vs MARGINAL" classification.
/// (Mirror of the value in `main.rs` — kept private here so the renderer
/// can label every saved HTML without the caller having to pass it in.)
const RENDER_STRICT_MARGIN: f64 = 1e-4;

static IMAGE_COUNTER: AtomicUsize = AtomicUsize::new(0);

const TILE_COLORS: [&str; 6] = [
    "#d62728", // 0 triple        - red
    "#2ca02c", // 1 corner        - green
    "#1f77b4", // 2 double        - blue
    "#ff7f0e", // 3 single        - orange
    "#9467bd", // 4 empty_double  - purple
    "#17becf", // 5 empty_single  - cyan
];

const TILE_NAMES: [&str; 6] = [
    "triple (1x1x3)",
    "corner (L)",
    "double (1x1x2)",
    "single (1x1x1)",
    "empty double",
    "empty single",
];

/// Build the standalone HTML string for a length-aware placement list, with
/// the legacy convention that index >= 4 is hollow. Used by the hot loop.
fn build_html(pls: &[&Placement], title: &str) -> String {
    let marked: Vec<(&Placement, bool)> =
        pls.iter().enumerate().map(|(i, &p)| (p, i >= 4)).collect();
    build_html_marked(&marked, title)
}

/// Build the HTML with an explicit per-placement hollow flag. Lets callers
/// opt out of the index-keyed convention used by `build_html`.
fn build_html_marked(pls: &[(&Placement, bool)], title: &str) -> String {
    // Build the JS tile array literal: [{color:"#...", cells:[[x,y,z], ...]}, ...]
    let mut tiles_js = String::from("[");
    for (i, (tile, is_empty)) in pls.iter().enumerate() {
        if i > 0 {
            tiles_js.push(',');
        }
        let color = TILE_COLORS[i.min(TILE_COLORS.len() - 1)];
        write!(
            tiles_js,
            "{{color:\"{}\",empty:{},cells:[",
            color, is_empty
        )
        .unwrap();
        for (j, &(x, y, z)) in tile.cells.iter().enumerate() {
            if j > 0 {
                tiles_js.push(',');
            }
            write!(tiles_js, "[{},{},{}]", x, y, z).unwrap();
        }
        tiles_js.push_str("]}");
    }
    tiles_js.push(']');

    // Legend rows — one per actually-present tile.
    let mut legend_rows = String::new();
    for (i, (tile, _is_empty)) in pls.iter().enumerate() {
        let color = TILE_COLORS[i.min(TILE_COLORS.len() - 1)];
        let name = TILE_NAMES.get(i).copied().unwrap_or("tile");
        write!(
            legend_rows,
            r#"<div class="row"><span class="sw" style="background:{}"></span>{}: {} ({:.0} g)</div>
"#,
            color, i, name, tile.mass
        )
        .unwrap();
    }

    // Stability outcome — three-way classification:
    //   STRICT   — stable even with the support polygon eroded by RENDER_STRICT_MARGIN
    //   MARGINAL — stable only at margin = 0 (COM exactly on a support edge)
    //   UNSTABLE — not stable at margin = 0 either
    // Two LP calls; cells outside 0..=2 mean this is a "showcase" render where
    // stability isn't meaningful (e.g. pieces_overview), so we suppress the
    // label entirely.
    let placement_refs: Vec<&Placement> = pls.iter().map(|(p, _)| *p).collect();
    let cells_in_grid = placement_refs.iter().all(|p| {
        p.cells.iter().all(|&(x, y, z)| x <= 2 && y <= 2 && z <= 2)
    });
    let stab_label = if !cells_in_grid {
        "showcase"
    } else if check_stability_quick(&placement_refs, RENDER_STRICT_MARGIN).is_stable() {
        "STRICT"
    } else if check_stability_quick(&placement_refs, 0.0).is_stable() {
        "MARGINAL"
    } else {
        "UNSTABLE"
    };

    // Front and side silhouette masks (computed across all placements regardless
    // of hollow/solid — the hollow pieces don't contribute to the masks because
    // their `front_mask`/`side_mask` are still set; we'll suppress the cards
    // when rendering hollow pieces by using only the solid placements).
    // Game rule: hollow pieces don't count towards the silhouette. So filter:
    let mut front_mask: u16 = 0;
    let mut side_mask: u16 = 0;
    for (p, is_hollow) in pls {
        if *is_hollow { continue; }
        front_mask |= p.front_mask;
        side_mask |= p.side_mask;
    }

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{title}</title>
<style>
  html, body {{ margin: 0; height: 100%; overflow: hidden; background: #f0f0f0;
                font-family: sans-serif; }}
  #legend {{ position: absolute; top: 12px; right: 12px;
             background: rgba(255,255,255,0.94); padding: 10px 14px;
             border-radius: 6px; font-size: 12px; line-height: 1.5;
             border: 1px solid #bbb; box-shadow: 0 1px 4px rgba(0,0,0,0.08);
             max-width: 280px; }}
  #legend h3 {{ margin: 0 0 6px 0; font-size: 13px; }}
  #legend .row {{ white-space: nowrap; }}
  #legend .sw {{ display: inline-block; width: 12px; height: 12px;
                 vertical-align: middle; margin-right: 6px;
                 border: 1px solid #333; }}
  #legend .stab {{ font-family: monospace; font-size: 11px; }}
  #hint {{ position: absolute; bottom: 10px; left: 12px;
           font-size: 11px; color: #555; }}
</style>
</head>
<body>
<div id="legend">
  <h3>Tiles</h3>
  {legend_rows}
  <h3 style="margin-top:10px">Stability</h3>
  <div class="stab">{stab_label}</div>
</div>
<div id="hint">drag = rotate &middot; scroll = zoom &middot; right-drag = pan</div>
<script type="importmap">
{{
  "imports": {{
    "three": "https://unpkg.com/three@0.160.0/build/three.module.js",
    "three/addons/": "https://unpkg.com/three@0.160.0/examples/jsm/"
  }}
}}
</script>
<script type="module">
import * as THREE from 'three';
import {{ OrbitControls }} from 'three/addons/controls/OrbitControls.js';

const TILES = {tiles_js};

const scene = new THREE.Scene();
scene.background = new THREE.Color(0xf0f0f0);

const camera = new THREE.PerspectiveCamera(
  40, window.innerWidth / window.innerHeight, 0.1, 100);
camera.position.set(6.5, 5.5, 6.5);

const renderer = new THREE.WebGLRenderer({{ antialias: true }});
renderer.setSize(window.innerWidth, window.innerHeight);
// Cap device pixel ratio: on a retina display devicePixelRatio is 2 or 3,
// which renders 4-9× the pixels of a 1.5× cap with no perceptible quality
// loss for these scenes. Big GPU win.
renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5));
document.body.appendChild(renderer.domElement);

const controls = new OrbitControls(camera, renderer.domElement);
controls.target.set(1.5, 1.5, 1.5);
// No damping — paired with the "render only on change" loop below, this makes
// the iframe near-zero-cost when idle. Damping would require a constant rAF
// to drive the inertia.
controls.enableDamping = false;
controls.update();

scene.add(new THREE.AmbientLight(0xffffff, 0.55));
const sun = new THREE.DirectionalLight(0xffffff, 0.9);
sun.position.set(6, 10, 8);
scene.add(sun);
const fill = new THREE.DirectionalLight(0xffffff, 0.25);
fill.position.set(-5, 4, -3);
scene.add(fill);

// Ground plate at z = 0.
const groundGeo = new THREE.PlaneGeometry(6, 6);
const groundMat = new THREE.MeshLambertMaterial({{
  color: 0xdddddd, side: THREE.DoubleSide
}});
const ground = new THREE.Mesh(groundGeo, groundMat);
ground.rotation.x = -Math.PI / 2;
ground.position.set(1.5, 0, 1.5);
scene.add(ground);
const grid = new THREE.GridHelper(3, 3, 0x666666, 0x999999);
grid.position.set(1.5, 0.002, 1.5);
scene.add(grid);

// Cubes. Our coords have z up; three.js has y up, so map (x, y, z) -> (x, z, y).
const SCALE = 0.98;             // drawn extent of a unit cell (gap between cells)
const HALF = SCALE / 2;         // half-side
const HOLE_R = 0.36;            // circular hole radius on each face
const HOLE_SEGS = 48;           // smoothness of the hole circle

// Solid cells: textbook BoxGeometry with thin black edges.
const cubeGeo = new THREE.BoxGeometry(SCALE, SCALE, SCALE);
const edgeGeo = new THREE.EdgesGeometry(cubeGeo);
const edgeMat = new THREE.LineBasicMaterial({{ color: 0x222222 }});

// Hollow cells: 6 square-with-circular-hole panels, one per cube face.
// ShapeGeometry triangulates a 2D shape (with a circular hole as a Path),
// living in the XY plane with normal +Z; we rotate copies onto each face.
const panelShape = new THREE.Shape();
panelShape.moveTo(-HALF, -HALF);
panelShape.lineTo( HALF, -HALF);
panelShape.lineTo( HALF,  HALF);
panelShape.lineTo(-HALF,  HALF);
panelShape.lineTo(-HALF, -HALF);
const panelHole = new THREE.Path();
panelHole.absarc(0, 0, HOLE_R, 0, Math.PI * 2, false);
panelShape.holes.push(panelHole);
const panelGeo = new THREE.ShapeGeometry(panelShape, HOLE_SEGS);

function addSolidCell(x, y, z, mat) {{
  const cube = new THREE.Mesh(cubeGeo, mat);
  cube.position.set(x + 0.5, z + 0.5, y + 0.5);
  scene.add(cube);
  const edges = new THREE.LineSegments(edgeGeo, edgeMat);
  edges.position.copy(cube.position);
  scene.add(edges);
}}

function addHollowCell(x, y, z, color) {{
  // DoubleSide so panels are visible from inside the hole too. No edges.
  // Lower opacity makes the "see-through" nature of hollow blocks visually
  // obvious — you can see other pieces (and the reference cards) through them.
  const mat = new THREE.MeshLambertMaterial({{
    color: color, side: THREE.DoubleSide,
    transparent: true, opacity: 0.55,
  }});
  // Three.js position is (world_x, world_z, world_y).
  const px = x + 0.5;
  const py = z + 0.5;
  const pz = y + 0.5;

  const faces = [
    // [rotation_x, rotation_y, offset_x, offset_y, offset_z]
    [0,            0,           0,        0,        HALF ], // +Z face
    [Math.PI,      0,           0,        0,       -HALF ], // -Z face
    [0,            Math.PI / 2, HALF,     0,        0    ], // +X face
    [0,           -Math.PI / 2,-HALF,     0,        0    ], // -X face
    [-Math.PI / 2, 0,           0,        HALF,     0    ], // +Y face
    [ Math.PI / 2, 0,           0,       -HALF,     0    ], // -Y face
  ];
  for (const [rx, ry, ox, oy, oz] of faces) {{
    const panel = new THREE.Mesh(panelGeo, mat);
    panel.rotation.x = rx;
    panel.rotation.y = ry;
    panel.position.set(px + ox, py + oy, pz + oz);
    scene.add(panel);
  }}
}}

for (const tile of TILES) {{
  const mat = new THREE.MeshLambertMaterial({{ color: tile.color }});
  for (const [x, y, z] of tile.cells) {{
    if (tile.empty) {{
      addHollowCell(x, y, z, tile.color);
    }} else {{
      addSolidCell(x, y, z, mat);
    }}
  }}
}}

// Front and side reference cards in the back of the scene — same silhouettes
// as the printed Project Cube cards. Drawn on a 2D canvas, applied as a
// CanvasTexture to a PlaneGeometry, rotated to face the camera direction.
const FRONT_MASK = {front_mask};
const SIDE_MASK  = {side_mask};

function makeCardTexture(mask, mirrorColumns) {{
  const canvas = document.createElement('canvas');
  const SIZE = 256;
  canvas.width = canvas.height = SIZE;
  const ctx = canvas.getContext('2d');
  ctx.fillStyle = '#fffbf0';
  ctx.fillRect(0, 0, SIZE, SIZE);
  ctx.strokeStyle = '#5a4a30';
  ctx.lineWidth = 5;
  ctx.strokeRect(8, 8, SIZE - 16, SIZE - 16);
  const cellSize = (SIZE - 32) / 3;
  const off = 16;
  ctx.fillStyle = '#3a3a3a';
  for (let z = 0; z < 3; z++) {{
    for (let a = 0; a < 3; a++) {{
      const bit = a * 3 + z;
      if ((mask >> bit) & 1) {{
        const aDisp = mirrorColumns ? (2 - a) : a;
        const x = off + aDisp * cellSize;
        const y = off + (2 - z) * cellSize;
        ctx.fillRect(x + 2, y + 2, cellSize - 4, cellSize - 4);
      }}
    }}
  }}
  const tex = new THREE.CanvasTexture(canvas);
  tex.needsUpdate = true;
  return tex;
}}

if (FRONT_MASK !== 0 || SIDE_MASK !== 0) {{
  const cardSpan = 3.0;
  const cardGeo = new THREE.PlaneGeometry(cardSpan, cardSpan);
  // Front card: behind the cube along Z, facing +Z (toward camera).
  const frontMat = new THREE.MeshBasicMaterial({{
    map: makeCardTexture(FRONT_MASK, false),
    side: THREE.DoubleSide,
    transparent: true,
    opacity: 0.93,
  }});
  const frontCard = new THREE.Mesh(cardGeo, frontMat);
  frontCard.position.set(1.5, 1.5, -0.6);
  scene.add(frontCard);
  // Side card: behind the cube along X, facing +X. Rotated -90° around Y so
  // its texture's "right" axis maps to world +Z (= world y, the side axis).
  const sideMat = new THREE.MeshBasicMaterial({{
    map: makeCardTexture(SIDE_MASK, true),
    side: THREE.DoubleSide,
    transparent: true,
    opacity: 0.93,
  }});
  const sideCard = new THREE.Mesh(cardGeo, sideMat);
  sideCard.rotation.y = -Math.PI / 2;
  sideCard.position.set(-0.6, 1.5, 1.5);
  scene.add(sideCard);
}}

function render() {{ renderer.render(scene, camera); }}
controls.addEventListener('change', render);
window.addEventListener('resize', () => {{
  camera.aspect = window.innerWidth / window.innerHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(window.innerWidth, window.innerHeight);
  render();
}});
// One initial render once everything is wired up.
render();
</script>
</body>
</html>
"##,
        title = title,
        legend_rows = legend_rows,
        stab_label = stab_label,
        tiles_js = tiles_js,
        front_mask = front_mask,
        side_mask = side_mask,
    )
}

/// Hot-loop renderer: writes `dir/cube_NNNNN.html` and respects `MAX_IMAGES`.
///
/// Returns `Some(path)` if the file was written, or `None` once `MAX_IMAGES`
/// has been reached.
pub fn save_html(pls: &Vec<&Placement>, dir: &str) -> Option<String> {
    let n = IMAGE_COUNTER.fetch_add(1, Ordering::Relaxed);
    if n >= MAX_IMAGES {
        return None;
    }
    std::fs::create_dir_all(dir).expect("create image dir");
    let path = format!("{}/cube_{:05}.html", dir, n);
    let title = format!("cube view #{:05}", n);
    let html = build_html(pls.as_slice(), &title);
    let file = File::create(&path).expect("create html file");
    let mut w = BufWriter::new(file);
    w.write_all(html.as_bytes()).unwrap();
    w.flush().unwrap();
    Some(path)
}

/// Named-output renderer: writes to a specific path, no global counter or
/// cap. Use for blog-post examples and other deliberate one-off renders.
pub fn save_html_to(pls: &[&Placement], path: &str, title: &str) {
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).expect("create parent dir");
        }
    }
    let html = build_html(pls, title);
    let file = File::create(path).expect("create html file");
    let mut w = BufWriter::new(file);
    w.write_all(html.as_bytes()).unwrap();
    w.flush().unwrap();
}

/// Like `save_html_to`, but with an explicit per-placement "is hollow" flag —
/// hollow rendering is decoupled from slice index, so callers can put the
/// hollow piece anywhere in the list.
pub fn save_html_to_marked(pls: &[(&Placement, bool)], path: &str, title: &str) {
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).expect("create parent dir");
        }
    }
    let html = build_html_marked(pls, title);
    let file = File::create(path).expect("create html file");
    let mut w = BufWriter::new(file);
    w.write_all(html.as_bytes()).unwrap();
    w.flush().unwrap();
}
