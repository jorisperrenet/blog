//! Render the compatibility graph (D4-distinct silhouettes; edge iff every
//! orientation cross-product entry has STABLE > 0 in both directions) as a
//! self-contained interactive HTML file using vis-network from a CDN.
//!
//! Each node is a 3x3 silhouette icon (inline SVG), edges connect compatible
//! cards. Open `compat_graph.html` in a browser, drag nodes to reorganise,
//! scroll to zoom.
//!
//! Run with `cargo run --release --bin graph_viz`. Writes `compat_graph.html`
//! to the working directory.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

// -------- silhouette helpers (same convention as view_counts.txt) --------

fn parse_grid(s: &str) -> u16 {
    let rows: Vec<&str> = s.split('|').collect();
    assert_eq!(rows.len(), 3);
    let mut mask = 0u16;
    for (i, row) in rows.iter().enumerate() {
        let z = 2 - i;
        let chars: Vec<char> = row.chars().collect();
        assert_eq!(chars.len(), 3);
        for (a, c) in chars.iter().enumerate() {
            if *c == '#' {
                mask |= 1u16 << (a * 3 + z);
            }
        }
    }
    mask
}

fn render(mask: u16) -> String {
    let mut s = String::with_capacity(11);
    for z in (0..3).rev() {
        for a in 0..3 {
            let bit = a * 3 + z;
            s.push(if (mask >> bit) & 1 == 1 { '#' } else { '.' });
        }
        if z > 0 {
            s.push('|');
        }
    }
    s
}

fn rotate90(mask: u16) -> u16 {
    let mut new_mask = 0u16;
    for a in 0..3u16 {
        for z in 0..3u16 {
            if (mask >> (a * 3 + z)) & 1 == 1 {
                new_mask |= 1u16 << (z * 3 + (2 - a));
            }
        }
    }
    new_mask
}

fn flip_h(mask: u16) -> u16 {
    let mut new_mask = 0u16;
    for a in 0..3u16 {
        for z in 0..3u16 {
            if (mask >> (a * 3 + z)) & 1 == 1 {
                new_mask |= 1u16 << ((2 - a) * 3 + z);
            }
        }
    }
    new_mask
}

fn d4_orbit(mask: u16) -> Vec<u16> {
    let mut set: HashSet<u16> = HashSet::new();
    let mut cur = mask;
    for _ in 0..4 {
        set.insert(cur);
        set.insert(flip_h(cur));
        cur = rotate90(cur);
    }
    let mut v: Vec<u16> = set.into_iter().collect();
    v.sort();
    v
}

fn canonical(mask: u16) -> u16 {
    *d4_orbit(mask).iter().min().unwrap()
}

// -------- view_counts.txt loader --------

fn read_view_counts(path: &str) -> HashMap<(u16, u16), (u64, u64)> {
    let file = File::open(path).unwrap_or_else(|e| panic!("open {}: {}", path, e));
    let mut map = HashMap::new();
    for line in BufReader::new(file).lines() {
        let line = line.expect("read line");
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let strict: u64 = match parts.next().and_then(|s| s.parse().ok()) {
            Some(v) => v,
            None => continue,
        };
        let marginal: u64 = match parts.next().and_then(|s| s.parse().ok()) {
            Some(v) => v,
            None => continue,
        };
        let front = parts.next().expect("front view");
        let side = parts.next().expect("side view");
        map.insert((parse_grid(front), parse_grid(side)), (strict, marginal));
    }
    map
}

/// Pair (i, j) is compatible iff every (orient_i, orient_j) view in either
/// direction has STABLE > 0. Returns the per-edge min STABLE / marginal at
/// that minimum across the cross-product (used to compute the subset's
/// weakest link in the JS sidebar).
fn pair_min_stable(
    oi: &[u16],
    oj: &[u16],
    vc: &HashMap<(u16, u16), (u64, u64)>,
) -> Option<(u64, u64)> {
    let mut min_s: u64 = u64::MAX;
    let mut min_m: u64 = 0;
    for &fi in oi {
        for &fj in oj {
            for &(a, b) in &[(fi, fj), (fj, fi)] {
                let (s, m) = vc.get(&(a, b)).copied().unwrap_or((0, 0));
                if s == 0 {
                    return None; // not compatible at all
                }
                if s < min_s {
                    min_s = s;
                    min_m = m;
                }
            }
        }
    }
    Some((min_s, min_m))
}

// -------- silhouette as inline SVG data URL --------

/// Render `mask` as an SVG icon and return a `data:image/svg+xml;utf8,...`
/// data URL suitable for use as a vis-network node image.
fn silhouette_data_url(mask: u16) -> String {
    let cell = 16;
    let pad = 2;
    let size = cell * 3 + pad * 2;
    let mut svg = String::new();
    write!(
        svg,
        r#"<svg xmlns='http://www.w3.org/2000/svg' width='{size}' height='{size}' viewBox='0 0 {size} {size}'>"#,
        size = size
    )
    .unwrap();
    write!(
        svg,
        r#"<rect width='100%' height='100%' fill='white' stroke='%23222' stroke-width='1.5'/>"#
    )
    .unwrap();
    for z in 0..3u32 {
        for a in 0..3u32 {
            let bit = a * 3 + z;
            if (mask >> bit) & 1 == 1 {
                let x = pad + a as i32 * cell;
                // top row = z = 2, so flip z for screen y.
                let y = pad + (2 - z as i32) * cell;
                write!(
                    svg,
                    r#"<rect x='{x}' y='{y}' width='{cell}' height='{cell}' fill='%23333'/>"#,
                    x = x,
                    y = y,
                    cell = cell
                )
                .unwrap();
            }
        }
    }
    svg.push_str("</svg>");
    // # and # already %23-escaped above. Spaces don't appear (single-quoted attrs).
    format!("data:image/svg+xml;utf8,{}", svg)
}

// -------- main --------

fn main() {
    // 1. D4-distinct silhouettes.
    let canons: Vec<u16> = {
        let mut set: HashSet<u16> = HashSet::new();
        for mask in 0u16..512u16 {
            set.insert(canonical(mask));
        }
        let mut v: Vec<u16> = set.into_iter().collect();
        v.sort();
        v
    };
    let n = canons.len();
    eprintln!("D4-distinct silhouettes: {}", n);
    let orientations: Vec<Vec<u16>> = canons.iter().map(|&m| d4_orbit(m)).collect();

    // 2. Read view counts and compute edges.
    let view_counts_path = std::env::var("PROJECT_CUBE_VIEW_COUNTS")
        .unwrap_or_else(|_| "view_counts.txt".to_string());
    eprintln!("reading view counts from {}", view_counts_path);
    let vc = read_view_counts(&view_counts_path);
    eprintln!("loaded {} view-pair entries", vc.len());

    // Edges with their per-pair min STABLE / min MARGINAL.
    let mut edges: Vec<(usize, usize, u64, u64)> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            if let Some((ms, mm)) = pair_min_stable(&orientations[i], &orientations[j], &vc) {
                edges.push((i, j, ms, mm));
            }
        }
    }
    eprintln!("{} compatibility edges", edges.len());

    // Per-node degree, for sizing/labeling.
    let mut degree = vec![0usize; n];
    for &(i, j, _, _) in &edges {
        degree[i] += 1;
        degree[j] += 1;
    }

    // 3a. Compact-grid placement for degree-0 nodes (so they don't drift
    //     all over the canvas and bloat the viewport).
    let isolated: Vec<usize> = (0..n).filter(|&i| degree[i] == 0).collect();
    let cols: i32 = 14;
    let spacing: i32 = 48;
    let grid_x0: i32 = -(cols * spacing) / 2;
    let grid_y0: i32 = 320; // just below the main connected component
    let mut grid_pos: HashMap<usize, (i32, i32)> = HashMap::new();
    for (k, &idx) in isolated.iter().enumerate() {
        let row = (k as i32) / cols;
        let col = (k as i32) % cols;
        grid_pos.insert(idx, (grid_x0 + col * spacing, grid_y0 + row * spacing));
    }
    eprintln!(
        "{} isolated (degree 0) nodes pinned to a {}-column grid",
        isolated.len(),
        cols
    );

    // 3b. Build the JS data literals.
    let mut nodes_js = String::from("[");
    for (i, &mask) in canons.iter().enumerate() {
        if i > 0 {
            nodes_js.push(',');
        }
        let url = silhouette_data_url(mask);
        let title = render(mask);
        if let Some(&(x, y)) = grid_pos.get(&i) {
            // Pinned isolated node.
            write!(
                nodes_js,
                "{{id:{i},image:\"{url}\",shape:\"image\",size:14,\
                 title:\"{title} (deg 0)\",x:{x},y:{y},fixed:true,physics:false}}",
                i = i, url = url, title = title, x = x, y = y
            ).unwrap();
        } else {
            write!(
                nodes_js,
                "{{id:{i},image:\"{url}\",shape:\"image\",size:18,\
                 title:\"{title} (deg {deg})\"}}",
                i = i, url = url, title = title, deg = degree[i]
            ).unwrap();
        }
    }
    nodes_js.push(']');

    let mut edges_js = String::from("[");
    for (k, &(i, j, ms, mm)) in edges.iter().enumerate() {
        if k > 0 {
            edges_js.push(',');
        }
        write!(
            edges_js,
            "{{id:{k},from:{i},to:{j},minStable:{ms},minMarginal:{mm}}}",
            k = k,
            i = i,
            j = j,
            ms = ms,
            mm = mm
        )
        .unwrap();
    }
    edges_js.push(']');

    // 4. Write the standalone HTML.
    let path_owned = std::env::var("PROJECT_CUBE_GRAPH_OUTPUT")
        .unwrap_or_else(|_| "compat_graph.html".to_string());
    let path: &str = &path_owned;
    let file = File::create(path).expect("create html");
    let mut w = BufWriter::new(file);
    let html = format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Compatibility graph</title>
<script src="https://unpkg.com/vis-network/standalone/umd/vis-network.min.js"></script>
<style>
  html, body {{ margin: 0; height: 100%; font-family: sans-serif; background: #f5f5f5; }}
  #app {{ display: flex; height: 100vh; }}
  #graph-wrap {{ flex: 1; min-width: 0; position: relative; }}
  #graph {{ width: 100%; height: 100%; }}
  /* On narrow viewports also shrink the sidebar's internals so the
     fluid width still has room. */
  @media (max-width: 900px) {{
    #sidebar {{ padding: 10px 12px; font-size: 12px; }}
    #sidebar h3 {{ font-size: 12px; }}
    #status {{ font-size: 11px; padding: 6px 8px; }}
    .sel-item img {{ width: 32px; height: 32px; }}
  }}
  @media (max-width: 600px) {{
    .sel-item img {{ width: 26px; height: 26px; }}
  }}
  #header {{ position: absolute; top: 8px; left: 12px; right: 12px;
             background: rgba(255,255,255,0.92); padding: 8px 12px;
             border: 1px solid #ccc; border-radius: 6px; font-size: 12px;
             pointer-events: none; line-height: 1.4; z-index: 10; }}
  /* Fluid width: 150 px floor for very narrow screens, 30vw target,
     320 px ceiling. Smooth across viewport sizes — no brittle media-query
     breakpoints needed. */
  #sidebar {{ width: clamp(150px, 30vw, 320px); padding: 14px; background: #fff;
              border-left: 1px solid #ccc; overflow-y: auto;
              font-size: 13px; line-height: 1.5; }}
  #sidebar h3 {{ margin: 0 0 8px 0; font-size: 14px; }}
  #status {{ padding: 8px 10px; border-radius: 4px; background: #f5f5f5;
             border: 1px solid #ddd; margin-bottom: 10px; font-size: 12px; }}
  #status.clique {{ background: #e6f7e6; border-color: #5cb85c; }}
  #status.broken {{ background: #fff3cd; border-color: #ddb44c; }}
  #status.empty  {{ background: #f5f5f5; border-color: #ccc; color: #666; }}
  #selected-list {{ display: flex; flex-wrap: wrap; gap: 4px;
                    margin-bottom: 10px; }}
  .sel-item {{ position: relative; display: inline-block; }}
  .sel-item img {{ display: block; border: 1px solid #888; cursor: pointer;
                   width: 44px; height: 44px; }}
  .sel-item .x {{ position: absolute; top: -6px; right: -6px;
                  width: 16px; height: 16px; line-height: 14px;
                  text-align: center; background: #d9534f; color: #fff;
                  border-radius: 50%; font-size: 11px; font-weight: bold;
                  cursor: pointer; user-select: none; }}
  #missing-list {{ font-family: monospace; font-size: 11px; color: #a05a00;
                   max-height: 200px; overflow-y: auto;
                   background: #fffaf0; padding: 6px 8px; border-radius: 4px;
                   border: 1px solid #ddb44c; }}
  #clear {{ background: #fff; border: 1px solid #888; padding: 4px 10px;
            border-radius: 3px; cursor: pointer; font-size: 12px; }}
  #clear:hover {{ background: #f5f5f5; }}
  .label-pat {{ font-family: monospace; font-size: 11px; color: #555; }}
  .stab-line {{ font-family: monospace; font-size: 12px; }}
</style>
</head>
<body>
<div id="app">
  <div id="graph-wrap">
    <div id="header">
      <strong>Compatibility graph</strong> — {n} D4-distinct silhouettes,
      {edge_count} edges. <strong>Click nodes to add/remove from a subset</strong>.
    </div>
    <div id="graph"></div>
  </div>
  <div id="sidebar">
    <h3>Subset builder</h3>
    <div id="status" class="empty">Click a node to start a subset.</div>
    <div id="selected-list"></div>
    <button id="clear">Clear selection</button>
    <h3 style="margin-top:14px">Missing edges</h3>
    <div id="missing-list">(none — clique is valid)</div>
  </div>
</div>
<script>
const nodesData = {nodes_js};
const edgesData = {edges_js};
const nodes = new vis.DataSet(nodesData);
const edges = new vis.DataSet(edgesData);

const NODE_COLORS_NORMAL   = {{ border: 'rgba(0,0,0,0)', background: '#fff' }};
const NODE_COLORS_SELECTED = {{ border: '#ff7b00',        background: '#fff3e0' }};
const NODE_COLORS_ADDABLE  = {{ border: '#3fa84a',        background: '#e6f7e6' }};
const NODE_COLORS_DISABLED = {{ border: '#cfd3dc',        background: '#f4f5f7' }};
const EDGE_COLOR_NEUTRAL  = '#9aa3b2';
// Soft peach for "current connections" — edges with exactly one endpoint
// selected. Reads as "related to your subset" without competing with the
// fully-saturated orange of edges entirely inside the subset.
const EDGE_COLOR_HALF     = '#ffc9a0';
const EDGE_COLOR_SELECTED = '#ff7b00';
// Build adjacency map for fast "addable" computation (a node v is addable
// iff every selected node is in v's neighbour set).
const adjacency = {{}};
for (const e of edgesData) {{
  (adjacency[e.from] = adjacency[e.from] || new Set()).add(e.to);
  (adjacency[e.to]   = adjacency[e.to]   || new Set()).add(e.from);
}}

const network = new vis.Network(
  document.getElementById('graph'),
  {{ nodes, edges }},
  {{
    nodes: {{
      shape: 'image', size: 18, borderWidth: 0,
      color: NODE_COLORS_NORMAL,
      // Without this, image-shape nodes ignore borderWidth/color.border
      // entirely, which is why the orange "border" wasn't actually drawing.
      shapeProperties: {{ useBorderWithImage: true }},
    }},
    edges: {{ color: {{ color: EDGE_COLOR_NEUTRAL }}, smooth: false, width: 0.7 }},
    // BarnesHut has no rotational degrees of freedom (unlike forceAtlas2),
    // so the graph doesn't drift in a circle while it stabilizes.
    physics: {{
      solver: 'barnesHut',
      barnesHut: {{
        gravitationalConstant: -5500,
        centralGravity: 0.18,
        springLength: 140,
        springConstant: 0.035,
        damping: 0.4,
        avoidOverlap: 0.7,
      }},
      stabilization: {{ iterations: 280, fit: false }},
    }},
    interaction: {{
      hover: true, tooltipDelay: 100,
      selectable: false, // we manage selection ourselves
      zoomView: true, dragView: true,
      dragNodes: true,
    }},
  }}
);

// Freeze layout once stabilized + fit ONCE. Multiple fallbacks because
// `stabilizationIterationsDone` doesn't always fire reliably (especially
// on first-paint after iframe load).
let didInitialFit = false;
function ensureFit() {{
  if (didInitialFit) return;
  network.fit({{ animation: false }});
  didInitialFit = true;
}}
network.once('stabilizationIterationsDone', function () {{
  network.setOptions({{ physics: {{ enabled: false }} }});
  ensureFit();
}});
// Fallback 1: after the initial draw cycle.
network.once('afterDrawing', function () {{
  setTimeout(ensureFit, 50);
}});
// Fallback 2: hard timeout in case neither event fires fast enough.
setTimeout(function () {{
  network.setOptions({{ physics: {{ enabled: false }} }});
  ensureFit();
}}, 1500);

const selected = new Set();

function patFromTitle(t) {{
  // title is "row|row|row (deg N)" — strip suffix.
  return t.split(' (')[0];
}}

function updateView() {{
  const sel = Array.from(selected);
  const selSet = new Set(sel);

  // Edges: classify each as both/half/none, recolor.
  let withinCount = 0;
  let minStable = Infinity;
  let minMarginal = 0;
  const edgeUpdates = [];
  for (const e of edges.get()) {{
    const both = selSet.has(e.from) && selSet.has(e.to);
    const half = !both && (selSet.has(e.from) || selSet.has(e.to));
    // Use the shorthand string form for `color` — vis-network's update
    // path is more reliable than nested {{color: {{...}}}} on repeated calls.
    edgeUpdates.push({{
      id: e.id,
      color: both ? EDGE_COLOR_SELECTED
                  : (half ? EDGE_COLOR_HALF : EDGE_COLOR_NEUTRAL),
      width: both ? 2.5 : 0.7,
    }});
    if (both) {{
      withinCount++;
      if (e.minStable < minStable) {{
        minStable = e.minStable;
        minMarginal = e.minMarginal;
      }}
    }}
  }}
  edges.update(edgeUpdates);

  // Compute the "addable" set: nodes connected to every currently selected
  // node. Empty selection → no addable highlighting (everything is fair game).
  const addable = new Set();
  if (sel.length > 0) {{
    for (const n of nodes.get()) {{
      if (selSet.has(n.id)) continue;
      const nbrs = adjacency[n.id] || new Set();
      let ok = true;
      for (const s of sel) {{
        if (!nbrs.has(s)) {{ ok = false; break; }}
      }}
      if (ok) addable.add(n.id);
    }}
  }}

  // Nodes: orange border for selected, green border for addable. Non-addable
  // (when a selection exists) gets a mild opacity dim so it reads as "off".
  const nodeUpdates = [];
  for (const n of nodes.get()) {{
    const isSel = selSet.has(n.id);
    const isAdd = addable.has(n.id);
    let color, borderWidth, opacity;
    if (isSel) {{
      color = NODE_COLORS_SELECTED;
      borderWidth = 4;
      opacity = 1.0;
    }} else if (isAdd) {{
      color = NODE_COLORS_ADDABLE;
      borderWidth = 3;
      opacity = 1.0;
    }} else if (sel.length > 0) {{
      color = NODE_COLORS_NORMAL;
      borderWidth = 0;
      opacity = 0.55; // softer than the prior 0.25
    }} else {{
      color = NODE_COLORS_NORMAL;
      borderWidth = 0;
      opacity = 1.0;
    }}
    // Spread the color object so vis can't accidentally share state across
    // entries (defensive — has bitten me on update#2 before).
    nodeUpdates.push({{ id: n.id, borderWidth: borderWidth,
                        color: {{ ...color }}, opacity: opacity }});
  }}
  nodes.update(nodeUpdates);
  network.redraw();

  // Status panel.
  const status = document.getElementById('status');
  const expected = sel.length * (sel.length - 1) / 2;
  if (sel.length === 0) {{
    status.className = 'empty';
    status.textContent = 'Click a node to start a subset.';
  }} else if (sel.length === 1) {{
    status.className = 'empty';
    status.textContent = '1 node selected. Add another to start evaluating.';
  }} else if (withinCount === expected) {{
    status.className = 'clique';
    status.innerHTML = `<strong>Valid clique</strong> (${{sel.length}} nodes, `
      + `${{withinCount}} edges)<br>`
      + `<span class="stab-line">min STRICT = ${{minStable}}, `
      + `MARGINAL at that view = ${{minMarginal}}</span>`;
  }} else {{
    status.className = 'broken';
    status.innerHTML = `<strong>${{sel.length}} nodes</strong>, `
      + `${{withinCount}}/${{expected}} edges &mdash; `
      + `missing ${{expected - withinCount}} for a clique`;
  }}

  // Missing-edges list (only meaningful for size >= 2).
  const ml = document.getElementById('missing-list');
  if (sel.length < 2 || withinCount === expected) {{
    ml.textContent = sel.length < 2 ? '(select at least 2 nodes)'
                                    : '(none — clique is valid)';
  }} else {{
    const present = new Set();
    for (const e of edges.get()) {{
      if (selSet.has(e.from) && selSet.has(e.to)) {{
        present.add(e.from < e.to ? `${{e.from}}-${{e.to}}` : `${{e.to}}-${{e.from}}`);
      }}
    }}
    const lines = [];
    for (let a = 0; a < sel.length; a++) {{
      for (let b = a + 1; b < sel.length; b++) {{
        const i = sel[a], j = sel[b];
        const key = i < j ? `${{i}}-${{j}}` : `${{j}}-${{i}}`;
        if (!present.has(key)) {{
          const ni = nodes.get(i), nj = nodes.get(j);
          lines.push(`${{patFromTitle(ni.title)}}  ⟷  ${{patFromTitle(nj.title)}}`);
        }}
      }}
    }}
    ml.innerHTML = lines.join('<br>');
  }}

  // Selected list (with remove buttons).
  const sl = document.getElementById('selected-list');
  sl.innerHTML = '';
  sel.forEach(id => {{
    const n = nodes.get(id);
    const item = document.createElement('div');
    item.className = 'sel-item';
    item.innerHTML = `<img src="${{n.image}}" title="${{patFromTitle(n.title)}}">`
                   + `<span class="x" data-id="${{id}}">×</span>`;
    sl.appendChild(item);
  }});
  sl.querySelectorAll('.x').forEach(btn => {{
    btn.onclick = (ev) => {{
      ev.stopPropagation();
      selected.delete(parseInt(btn.dataset.id));
      updateView();
    }};
  }});
}}

network.on('click', function(params) {{
  // Fallback to a hit-test on the click position; vis sometimes misses the
  // node in `params.nodes` if the click landed on the image's transparent
  // area. `getNodeAt` works in DOM pixel coordinates.
  let id = params.nodes[0];
  if (id === undefined && params.pointer && params.pointer.DOM) {{
    id = network.getNodeAt(params.pointer.DOM);
  }}
  if (id !== undefined && id !== null) {{
    if (selected.has(id)) selected.delete(id);
    else selected.add(id);
    network.unselectAll(); // clear any internal selection state
    updateView();
  }}
}});

document.getElementById('clear').onclick = () => {{
  selected.clear();
  updateView();
}};

updateView();
</script>
</body>
</html>
"##,
        n = n,
        edge_count = edges.len(),
        nodes_js = nodes_js,
        edges_js = edges_js
    );
    w.write_all(html.as_bytes()).expect("write html");
    w.flush().unwrap();
    eprintln!("wrote {}", path);
}
