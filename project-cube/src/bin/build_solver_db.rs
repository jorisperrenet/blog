//! Walk the same enumeration as main.rs and build a JSON manifest mapping
//! every (front, side) silhouette pair (in the subset-allowed regime) to
//! one example tower, encoded compactly. Used by `blog/solver.html`.
//!
//! Output schema (one line of JSON, an array):
//!   [
//!     [front_mask, side_mask, strict_count, marginal_count,
//!      [[type, cells_mask], ...]],
//!     ...
//!   ]
//!
//!   - `type` is 0..=5: 0 triple, 1 corner, 2 double, 3 single,
//!                     4 hollow_double, 5 hollow_single
//!   - `cells_mask` packs the cells into 27 bits (bit `x*9 + 3y + z`).
//!
//! Run with `cargo run --release --bin build_solver_db`. Takes ~1 minute.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use project_cube::balance::{check_stability_quick, Placement};

const FULL_CELL_MASS: f64 = 150.0;
const HOLLOW_CELL_MASS: f64 = 50.0;
const STRICT_MARGIN: f64 = 1e-4;

fn enumerate_placements(
    base_shapes: &[Vec<(i32, i32, i32)>],
    mass: f64,
) -> Vec<Placement> {
    let mut by_mask: HashMap<u64, Vec<(u8, u8, u8)>> = HashMap::new();
    for x in 0..=2i32 {
        for y in 0..=2i32 {
            for z in 0..=2i32 {
                'shape: for shape in base_shapes {
                    let mut v = Vec::with_capacity(shape.len());
                    for cell in shape {
                        let tx = cell.0 + x;
                        let ty = cell.1 + y;
                        let tz = cell.2 + z;
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
    let mut placements: Vec<Placement> = by_mask
        .into_iter()
        .map(|(_, c)| Placement::new(c, mass))
        .collect();
    placements.sort_by_key(|p| p.occupy);
    placements
}

fn cells_mask_u32(cells: &[(u8, u8, u8)]) -> u32 {
    cells.iter().fold(0u32, |a, &(x, y, z)| {
        a | (1u32 << (x as u32 * 9 + 3 * y as u32 + z as u32))
    })
}

fn classify(cells: &[(u8, u8, u8)], hollow: bool) -> u8 {
    let n = cells.len();
    match (n, hollow) {
        (1, false) => 3,
        (1, true) => 5,
        (2, false) => 2,
        (2, true) => 4,
        (3, false) => {
            // triple = colinear (only one axis varies); corner = two axes vary.
            let xs: std::collections::HashSet<u8> =
                cells.iter().map(|c| c.0).collect();
            let ys: std::collections::HashSet<u8> =
                cells.iter().map(|c| c.1).collect();
            let zs: std::collections::HashSet<u8> =
                cells.iter().map(|c| c.2).collect();
            let varying = (xs.len() > 1) as u8 + (ys.len() > 1) as u8 + (zs.len() > 1) as u8;
            if varying == 1 { 0 } else { 1 }
        }
        _ => 99,
    }
}

#[inline]
fn unpack(p: Option<&Placement>) -> (u64, u16, u16) {
    match p {
        Some(p) => (p.occupy, p.front_mask, p.side_mask),
        None => (0u64, 0u16, 0u16),
    }
}

#[derive(Clone)]
struct StoredTower {
    placements: Vec<(u8, u32)>, // (type_id, cells_mask)
    is_strict: bool,
}

fn main() {
    eprintln!("building solver DB (full enumeration; ~1 minute)");

    // Same placement universe as main.rs.
    let single: Vec<Vec<(i32, i32, i32)>> = vec![vec![(0, 0, 0)]];
    let double: Vec<Vec<(i32, i32, i32)>> = vec![
        vec![(0, 0, 0), (1, 0, 0)],
        vec![(0, 0, 0), (0, 1, 0)],
        vec![(0, 0, 0), (0, 0, 1)],
    ];
    let triple: Vec<Vec<(i32, i32, i32)>> = vec![
        vec![(0, 0, 0), (1, 0, 0), (2, 0, 0)],
        vec![(0, 0, 0), (0, 1, 0), (0, 2, 0)],
        vec![(0, 0, 0), (0, 0, 1), (0, 0, 2)],
    ];
    let mut corner: Vec<Vec<(i32, i32, i32)>> = Vec::new();
    for &(da, db) in &[
        ((1, 0), (0, 1)), ((1, 0), (0, -1)),
        ((-1, 0), (0, 1)), ((-1, 0), (0, -1)),
    ] {
        corner.push(vec![(0, 0, 0), (da.0, da.1, 0), (da.0 + db.0, da.1 + db.1, 0)]);
        corner.push(vec![(0, 0, 0), (da.0, 0, da.1), (da.0 + db.0, 0, da.1 + db.1)]);
        corner.push(vec![(0, 0, 0), (0, da.0, da.1), (0, da.0 + db.0, da.1 + db.1)]);
    }
    let p_triple  = enumerate_placements(&triple,  3.0 * FULL_CELL_MASS);
    let p_corner  = enumerate_placements(&corner,  3.0 * FULL_CELL_MASS);
    let p_double  = enumerate_placements(&double,  2.0 * FULL_CELL_MASS);
    let p_single  = enumerate_placements(&single,        FULL_CELL_MASS);
    let p_hdouble = enumerate_placements(&double,  2.0 * HOLLOW_CELL_MASS);
    let p_hsingle = enumerate_placements(&single,        HOLLOW_CELL_MASS);
    eprintln!(
        "placement counts: {} {} {} {} {} {}",
        p_triple.len(), p_corner.len(), p_double.len(), p_single.len(),
        p_hdouble.len(), p_hsingle.len()
    );

    // Read view_counts.txt for the strict/marginal numbers per pair.
    let mut counts: HashMap<(u16, u16), (u64, u64)> = HashMap::new();
    {
        let f = File::open("view_counts.txt").expect("view_counts.txt");
        for line in BufReader::new(f).lines() {
            let line = line.unwrap();
            let s = line.trim();
            if s.is_empty() || s.starts_with('#') { continue; }
            let mut p = s.split_whitespace();
            let strict: u64 = match p.next().and_then(|x| x.parse().ok()) {
                Some(v) => v, None => continue,
            };
            let marginal: u64 = match p.next().and_then(|x| x.parse().ok()) {
                Some(v) => v, None => continue,
            };
            let parse_grid = |g: &str| -> u16 {
                let rows: Vec<&str> = g.split('|').collect();
                let mut m = 0u16;
                for (i, row) in rows.iter().enumerate() {
                    let z = 2 - i;
                    for (a, c) in row.chars().enumerate() {
                        if c == '#' { m |= 1u16 << (a * 3 + z); }
                    }
                }
                m
            };
            let f_mask = parse_grid(p.next().unwrap());
            let s_mask = parse_grid(p.next().unwrap());
            counts.insert((f_mask, s_mask), (strict, marginal));
        }
    }
    eprintln!("loaded {} view-pair counts", counts.len());

    // Run the full subset-allowed enumeration. For each non-overlapping
    // 6-(possibly-some-skipped)-tuple, compute (f, s), check stability,
    // and update the stored tower for that pair.
    let mut towers: HashMap<(u16, u16), StoredTower> = HashMap::new();
    let total_outer = p_triple.len() + 1;
    for (i_outer, p1_opt) in p_triple.iter().map(Some).chain(std::iter::once(None)).enumerate() {
        let (o1, f1, s1) = unpack(p1_opt);
        eprintln!("outer {}/{}: {}", i_outer + 1, total_outer,
                  if let Some(p) = p1_opt { format!("{:?}", p.cells) } else { "(skip triple)".to_string() });
        for p2_opt in p_corner.iter().map(Some).chain(std::iter::once(None)) {
            if let Some(p) = p2_opt { if o1 & p.occupy != 0 { continue; } }
            let (do2, df2, ds2) = unpack(p2_opt);
            let o2 = o1 | do2; let f2 = f1 | df2; let s2 = s1 | ds2;
            for p3_opt in p_double.iter().map(Some).chain(std::iter::once(None)) {
                if let Some(p) = p3_opt { if o2 & p.occupy != 0 { continue; } }
                let (do3, df3, ds3) = unpack(p3_opt);
                let o3 = o2 | do3; let f3 = f2 | df3; let s3 = s2 | ds3;
                for p4_opt in p_single.iter().map(Some).chain(std::iter::once(None)) {
                    if let Some(p) = p4_opt { if o3 & p.occupy != 0 { continue; } }
                    let (do4, df4, ds4) = unpack(p4_opt);
                    let o4 = o3 | do4;
                    let front = f3 | df4;
                    let side = s3 | ds4;
                    // Skip pairs we already know are strict — no need to scan
                    // for a "better" tower than a strict one.
                    if let Some(t) = towers.get(&(front, side)) {
                        if t.is_strict { continue; }
                    }

                    'p5: for p5_opt in p_hdouble.iter().map(Some).chain(std::iter::once(None)) {
                        if let Some(p) = p5_opt { if o4 & p.occupy != 0 { continue; } }
                        let do5 = p5_opt.map_or(0u64, |p| p.occupy);
                        let o5 = o4 | do5;
                        for p6_opt in p_hsingle.iter().map(Some).chain(std::iter::once(None)) {
                            if let Some(p) = p6_opt { if o5 & p.occupy != 0 { continue; } }

                            let mut active: Vec<&Placement> = Vec::with_capacity(6);
                            if let Some(p) = p1_opt { active.push(p); }
                            if let Some(p) = p2_opt { active.push(p); }
                            if let Some(p) = p3_opt { active.push(p); }
                            if let Some(p) = p4_opt { active.push(p); }
                            if let Some(p) = p5_opt { active.push(p); }
                            if let Some(p) = p6_opt { active.push(p); }
                            if active.is_empty() { continue; }

                            let strict_ok = check_stability_quick(&active, STRICT_MARGIN).is_stable();
                            let marg_ok = strict_ok || check_stability_quick(&active, 0.0).is_stable();
                            if !marg_ok { continue; }

                            let should_replace = match towers.get(&(front, side)) {
                                None => true,
                                Some(t) => strict_ok && !t.is_strict,
                            };
                            if !should_replace { continue; }

                            let mut data: Vec<(u8, u32)> = Vec::with_capacity(6);
                            let entries: [(Option<&Placement>, bool); 6] = [
                                (p1_opt, false),
                                (p2_opt, false),
                                (p3_opt, false),
                                (p4_opt, false),
                                (p5_opt, true),
                                (p6_opt, true),
                            ];
                            for (opt, hollow) in entries.iter() {
                                if let Some(p) = opt {
                                    let kind = classify(&p.cells, *hollow);
                                    data.push((kind, cells_mask_u32(&p.cells)));
                                }
                            }
                            towers.insert((front, side), StoredTower {
                                placements: data,
                                is_strict: strict_ok,
                            });
                            if strict_ok { break 'p5; }
                        }
                    }
                }
            }
        }
    }
    eprintln!("collected {} tower examples", towers.len());

    // Write JSON. Compact format: array of arrays.
    let path = "../static/project-cube/assets/solver_db.json";
    std::fs::create_dir_all("../static/project-cube/assets").expect("create assets dir");
    let f = File::create(path).expect("create solver_db.json");
    let mut w = BufWriter::new(f);
    write!(w, "[").unwrap();
    let mut keys: Vec<(u16, u16)> = towers.keys().copied().collect();
    keys.sort();
    let mut first = true;
    for (front, side) in keys {
        let tower = towers.get(&(front, side)).unwrap();
        let (strict, marginal) = counts.get(&(front, side)).copied().unwrap_or((0, 0));
        if !first { write!(w, ",").unwrap(); }
        first = false;
        write!(w, "[{},{},{},{},[", front, side, strict, marginal).unwrap();
        for (i, (kind, cells)) in tower.placements.iter().enumerate() {
            if i > 0 { write!(w, ",").unwrap(); }
            write!(w, "[{},{}]", kind, cells).unwrap();
        }
        write!(w, "]]").unwrap();
    }
    write!(w, "]").unwrap();
    w.flush().unwrap();
    eprintln!("wrote {}", path);
}
