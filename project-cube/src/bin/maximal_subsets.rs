//! Find every maximal subset of D4-distinct 3x3 silhouettes such that, for
//! every two distinct cards in the subset, every (front_orient, side_orient)
//! lookup in `view_counts.txt` (in either direction) has at least one
//! strictly-stable solution (STRICT > 0 in `view_counts.txt`).
//!
//! Procedure:
//! 1. Enumerate all 512 9-bit silhouettes; dedupe by D4 canonical form (the
//!    smallest mask in each orbit). That gives ~102 representatives.
//! 2. Build a compatibility graph: an edge between cards i and j iff every
//!    orientation cross-product entry has STABLE > 0 in both directions.
//! 3. Enumerate every maximal clique with Bron–Kerbosch (pivoting variant).
//! 4. For each maximal clique with ≥ 2 cards, scan every (i, j, orient_i,
//!    orient_j) and report the minimum STABLE plus the achieving 4-tuple.
//!
//! Output goes to stdout, sorted by clique size descending. Status to stderr.
//! Run with `cargo run --release --bin maximal_subsets`.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};

// -----------------------------------------------------------------------
//  3x3 silhouette helpers (bit (a*3 + z): a=column, z=row, z=0 = bottom).
//  This matches the convention used in main.rs's render_view + view_counts.txt.
// -----------------------------------------------------------------------

fn parse_grid(s: &str) -> u16 {
    let rows: Vec<&str> = s.split('|').collect();
    assert_eq!(rows.len(), 3, "expected 3 rows in {:?}", s);
    let mut mask = 0u16;
    for (i, row) in rows.iter().enumerate() {
        let z = 2 - i; // input row 0 = top = z=2
        let chars: Vec<char> = row.chars().collect();
        assert_eq!(chars.len(), 3, "expected 3 cols in {:?}", row);
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

/// 90° CW rotation in (column, row) coordinates: (a, z) -> (z, 2-a).
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

/// Mirror left/right: (a, z) -> (2-a, z).
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

// -----------------------------------------------------------------------
//  view_counts.txt loader.
// -----------------------------------------------------------------------

/// Read a "<id> <pattern>" file (the original 15 game cards) and return a
/// map from D4 canonical mask -> sorted list of game indices whose card has
/// that canonical. Multiple game cards mapping to the same canonical (i.e.
/// they're D4-duplicates of each other) all appear in the same vec.
fn read_game_indices(path: &str) -> HashMap<u16, Vec<u32>> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return HashMap::new(),
    };
    let mut map: HashMap<u16, Vec<u32>> = HashMap::new();
    for line in BufReader::new(file).lines() {
        let line = line.expect("read line");
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let id: u32 = match parts.next().and_then(|s| s.parse().ok()) {
            Some(v) => v,
            None => continue,
        };
        let pattern = match parts.next() {
            Some(p) => p,
            None => continue,
        };
        let canon = canonical(parse_grid(pattern));
        map.entry(canon).or_default().push(id);
    }
    for v in map.values_mut() {
        v.sort();
    }
    map
}

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

// -----------------------------------------------------------------------
//  Compatibility check.
// -----------------------------------------------------------------------

/// Cards i and j are compatible iff every orientation cross product has
/// STABLE > 0 in both (front, side) directions.
fn pair_compatible(
    oi: &[u16],
    oj: &[u16],
    vc: &HashMap<(u16, u16), (u64, u64)>,
) -> bool {
    for &fi in oi {
        for &fj in oj {
            if vc.get(&(fi, fj)).map_or(0, |&(s, _)| s) == 0 {
                return false;
            }
            if vc.get(&(fj, fi)).map_or(0, |&(s, _)| s) == 0 {
                return false;
            }
        }
    }
    true
}

// -----------------------------------------------------------------------
//  Bron–Kerbosch with pivot, on u128 bitmasks (n ≤ 128).
// -----------------------------------------------------------------------

fn bron_kerbosch(
    mut p: u128,
    mut x: u128,
    r: u128,
    neighbors: &[u128],
    out: &mut Vec<u128>,
) {
    if p == 0 && x == 0 {
        out.push(r);
        return;
    }
    // Pivot: vertex in P ∪ X with max |P ∩ N(u)|.
    let pool = p | x;
    let mut best = pool.trailing_zeros() as usize;
    let mut best_count: u32 = (p & neighbors[best]).count_ones();
    let mut bits = pool & (pool - 1);
    while bits != 0 {
        let v = bits.trailing_zeros() as usize;
        bits &= bits - 1;
        let count = (p & neighbors[v]).count_ones();
        if count > best_count {
            best_count = count;
            best = v;
        }
    }
    let mut candidates = p & !neighbors[best];
    while candidates != 0 {
        let v = candidates.trailing_zeros() as usize;
        let v_bit = 1u128 << v;
        candidates &= !v_bit;
        bron_kerbosch(
            p & neighbors[v],
            x & neighbors[v],
            r | v_bit,
            neighbors,
            out,
        );
        p &= !v_bit;
        x |= v_bit;
    }
}

// -----------------------------------------------------------------------
//  Main.
// -----------------------------------------------------------------------

fn main() {
    // 1. Enumerate all 512 silhouettes, dedupe by D4 canonical form.
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
    if n > 128 {
        panic!("too many cards for u128 bitmask: {}", n);
    }

    let orientations: Vec<Vec<u16>> = canons.iter().map(|&m| d4_orbit(m)).collect();

    // 2. Read view counts and build compatibility graph. The input file is
    // configurable via the PROJECT_CUBE_VIEW_COUNTS env var so the binary
    // can be run against either regime (subset-allowed or all-pieces).
    let vc_path = std::env::var("PROJECT_CUBE_VIEW_COUNTS")
        .unwrap_or_else(|_| "view_counts.txt".to_string());
    eprintln!("reading view counts from {}", vc_path);
    let vc = read_view_counts(&vc_path);
    eprintln!("loaded {} view-pair entries", vc.len());
    let game_indices = read_game_indices("cards_original.txt");
    eprintln!(
        "loaded {} game-card canonical entries from cards_original.txt",
        game_indices.len()
    );

    // Helper: format an index tag for a canonical mask.
    let tag = |canon: u16| -> String {
        match game_indices.get(&canon) {
            Some(ids) => {
                let s: Vec<String> = ids.iter().map(|x| x.to_string()).collect();
                format!("[{:>4}]", s.join(","))
            }
            None => "[    ]".to_string(),
        }
    };

    let mut neighbors: Vec<u128> = vec![0u128; n];
    let mut edges: u64 = 0;
    for i in 0..n {
        for j in (i + 1)..n {
            if pair_compatible(&orientations[i], &orientations[j], &vc) {
                neighbors[i] |= 1u128 << j;
                neighbors[j] |= 1u128 << i;
                edges += 1;
            }
        }
    }
    let max_edges = (n as u64) * (n as u64 - 1) / 2;
    eprintln!(
        "compatibility edges: {}/{} ({:.1}% density)",
        edges,
        max_edges,
        100.0 * edges as f64 / max_edges.max(1) as f64
    );

    // 3. Bron–Kerbosch.
    let initial_p: u128 = if n == 128 { !0u128 } else { (1u128 << n) - 1 };
    let mut cliques: Vec<u128> = Vec::new();
    bron_kerbosch(initial_p, 0, 0, &neighbors, &mut cliques);
    eprintln!("found {} maximal cliques", cliques.len());

    // 4. Drop singletons / empty cliques and sort by size descending.
    cliques.retain(|&c| c.count_ones() >= 2);
    cliques.sort_by_key(|&c| std::cmp::Reverse(c.count_ones()));

    println!(
        "# Maximal D4-distinct silhouette subsets where every pair of cards",
    );
    println!(
        "# (in any orientation, in either front/side direction) admits at least",
    );
    println!(
        "# one strictly-stable tower (STRICT > 0 in view_counts.txt).",
    );
    println!("#");
    println!(
        "# {} D4-distinct silhouettes; {} compatibility edges; {} maximal cliques (size >= 2).",
        n,
        edges,
        cliques.len()
    );
    println!();

    for &c in &cliques {
        let card_idx: Vec<usize> = (0..n).filter(|&i| c & (1u128 << i) != 0).collect();
        let size = card_idx.len();

        println!("subset (size {}):", size);
        for &i in &card_idx {
            println!("  {} {}", tag(canons[i]), render(canons[i]));
        }

        // Find min STABLE across the (i, j, orient_i, orient_j) cross-product.
        let mut min_stable: u64 = u64::MAX;
        let mut min_marginal: u64 = 0;
        let mut min_combo: Option<(u16, u16, u16, u16)> = None;
        for &i in &card_idx {
            for &j in &card_idx {
                if i == j {
                    continue;
                }
                for &oi in &orientations[i] {
                    for &oj in &orientations[j] {
                        let (st, mg) = vc.get(&(oi, oj)).copied().unwrap_or((0, 0));
                        if st < min_stable {
                            min_stable = st;
                            min_marginal = mg;
                            min_combo = Some((canons[i], canons[j], oi, oj));
                        }
                    }
                }
            }
        }
        if let Some((ci, cj, oi, oj)) = min_combo {
            println!(
                "  min STRICT = {}  (MARGINAL at this view = {})",
                min_stable, min_marginal
            );
            println!("  achieved by:");
            println!(
                "    front card {} (canonical {}) at orient {}",
                tag(ci),
                render(ci),
                render(oi)
            );
            println!(
                "    side  card {} (canonical {}) at orient {}",
                tag(cj),
                render(cj),
                render(oj)
            );
        }
        println!();
    }
}
