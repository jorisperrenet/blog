use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use rayon::prelude::*;

use project_cube::balance::{self, Placement};
#[allow(unused_imports)]
use project_cube::render;

/// Strictness margin (in cell-width units) used for the "strict" balance
/// classification. A 4-tuple counts as a *strict* solution iff at least one
/// empty-tile placement makes the tower stand with every contact face eroded
/// by this much on each edge — i.e. no tile balances exactly on a support
/// boundary. `1e-9` catches only mathematically-exact-on-edge cases (the user's
/// "barely balancing" example) while leaving every robustly stable tower alone.
const STRICT_MARGIN: f64 = 1e-4;

/// Runtime-configurable: env var `PROJECT_CUBE_ALL_PIECES` (1/true/yes) makes
/// every loop level mandatory. Default (false) lets the enumeration consider
/// skipping any piece, finding stable towers built from any subset of the 6.

/// Per-thread tally accumulated by the rayon `map`/`reduce`. A "solution" is
/// a distinct (triple, corner, double, single) 4-tuple for which at least one
/// placement of the two empty tiles makes the tower stand.
#[derive(Default)]
struct LocalCounts {
    /// Number of non-overlapping (triple, corner, double, single) 4-tuples seen.
    combos_seen: u64,
    /// 4-tuples that have a *strictly* stable empty-tile arrangement
    /// (every tile's COM strictly inside its eroded support).
    strict_solutions: u64,
    /// 4-tuples that have a *permissively* stable empty-tile arrangement
    /// (boundary-touching COM allowed).
    permissive_solutions: u64,
    /// (strict_count, permissive_count) per (front, side) view.
    views: HashMap<(u16, u16), (u64, u64)>,
}

impl LocalCounts {
    fn merge(mut self, other: LocalCounts) -> LocalCounts {
        self.combos_seen += other.combos_seen;
        self.strict_solutions += other.strict_solutions;
        self.permissive_solutions += other.permissive_solutions;
        for (k, (s, p)) in other.views {
            let entry = self.views.entry(k).or_insert((0, 0));
            entry.0 += s;
            entry.1 += p;
        }
        self
    }
}

/// Render a 9-bit view mask (bit at position `a*3 + z`, where `a` is x for
/// the front view and y for the side view) as 3 rows of 3 chars,
/// top row = z=2, separated by `|`.
fn render_view(mask: u16) -> String {
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

/// Pack a list of cells into a 27-bit occupancy mask. Cell (x, y, z) → bit x*9 + 3y + z.
fn positions_to_grid(cells: &[(u8, u8, u8)]) -> u64 {
    let mut grid = 0u64;
    for &(x, y, z) in cells {
        grid |= 1u64 << (x as u32 * 9 + 3 * y as u32 + z as u32);
    }
    grid
}

/// Build every distinct placement (deduped by occupied-cell bitmask) of one tile type.
fn enumerate_placements(base_shapes: &[Vec<(i32, i32, i32)>], mass: f64) -> Vec<Placement> {
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
                    let mask = positions_to_grid(&v);
                    by_mask.entry(mask).or_insert(v);
                }
            }
        }
    }
    let mut placements: Vec<Placement> = by_mask
        .into_iter()
        .map(|(_, cells)| Placement::new(cells, mass))
        .collect();
    placements.sort_by_key(|p| p.occupy);
    placements
}

/// Unpack an optional placement to its (occupy, front_mask, side_mask)
/// contribution. A `None` (skipped) piece contributes nothing.
#[inline]
fn unpack(p: Option<&Placement>) -> (u64, u16, u16) {
    match p {
        Some(p) => (p.occupy, p.front_mask, p.side_mask),
        None => (0u64, 0u16, 0u16),
    }
}

/// Iterate `ps` yielding each placement wrapped in `Some`, plus a trailing
/// `None` if `allow_skip` — i.e. "skipping this piece" becomes one more
/// option at this loop level when `ALL_PIECES_REQUIRED == false`.
fn iter_or_skip<'a>(
    ps: &'a [Placement],
    allow_skip: bool,
) -> impl Iterator<Item = Option<&'a Placement>> + 'a {
    let tail: Option<Option<&'a Placement>> = if allow_skip { Some(None) } else { None };
    ps.iter().map(Some).chain(tail)
}

fn main() {
    let all_pieces_required: bool = std::env::var("PROJECT_CUBE_ALL_PIECES")
        .map(|s| matches!(s.as_str(), "1" | "true" | "yes" | "TRUE"))
        .unwrap_or(false);
    let output_path: String = std::env::var("PROJECT_CUBE_OUTPUT")
        .unwrap_or_else(|_| "view_counts.txt".to_string());
    eprintln!(
        "ALL_PIECES_REQUIRED = {}, output = {}",
        all_pieces_required, output_path
    );

    // Clear out any HTML left over from a previous run so the cube_views
    // directory only contains renders from the current invocation.
    if let Ok(entries) = std::fs::read_dir("cube_views") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("html") {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    // ------------------------------------------------------------------
    //  Tile shape catalogues (orientations of each polycube as cell offsets).
    // ------------------------------------------------------------------
    let single = vec![vec![(0, 0, 0)]];

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

    // L-tromino in each of the 3 coordinate planes, with all 4 rotations.
    // Translations + dedup recover the remaining reflective orientations.
    let mut corner: Vec<Vec<(i32, i32, i32)>> = Vec::new();
    for &(da, db) in &[
        ((1, 0), (0, 1)),
        ((1, 0), (0, -1)),
        ((-1, 0), (0, 1)),
        ((-1, 0), (0, -1)),
    ] {
        corner.push(vec![(0, 0, 0), (da.0, da.1, 0), (da.0 + db.0, da.1 + db.1, 0)]);
        corner.push(vec![(0, 0, 0), (da.0, 0, da.1), (da.0 + db.0, 0, da.1 + db.1)]);
        corner.push(vec![(0, 0, 0), (0, da.0, da.1), (0, da.0 + db.0, da.1 + db.1)]);
    }

    let empty_single = single.clone();
    let empty_double = double.clone();

    // The four visible tiles are placed in the outer loops; the two empty tiles
    // are placed in the inner loops so that as soon as one stable empty
    // arrangement is found we can early-exit and count one solution.
    //
    //   placements[0] = triple        (1×1×3 full,  120 g)   — outer
    //   placements[1] = corner        (L-tromino,   120 g)
    //   placements[2] = double        (1×1×2 full,   80 g)
    //   placements[3] = single        (1×1×1 full,   40 g)   — last visible
    //   placements[4] = empty_double  (1×1×2 empty,  26 g)   — first invisible
    //   placements[5] = empty_single  (1×1×1 empty,  13 g)   — innermost
    let placements: Vec<Vec<Placement>> = vec![
        enumerate_placements(&triple, 450.0),
        enumerate_placements(&corner, 450.0),
        enumerate_placements(&double, 300.0),
        enumerate_placements(&single, 150.0),
        enumerate_placements(&empty_double, 100.0),
        enumerate_placements(&empty_single, 50.0),
    ];
    eprintln!(
        "placement counts: {:?}",
        placements.iter().map(|v| v.len()).collect::<Vec<_>>()
    );

    // ------------------------------------------------------------------
    //  Hot loop, parallel over `triple` placements.
    // ------------------------------------------------------------------
    let started = Instant::now();
    let done = AtomicU64::new(0);
    let allow_skip = !all_pieces_required;
    // Build the outer iteration set up-front so par_iter has a concrete slice.
    let outer: Vec<Option<&Placement>> = iter_or_skip(&placements[0], allow_skip).collect();
    let total_outer = outer.len() as u64;

    let summary = outer
        .par_iter()
        .map(|&p1_opt| {
            let mut local = LocalCounts::default();
            let (o1, f1, s1) = unpack(p1_opt);

            for p2_opt in iter_or_skip(&placements[1], allow_skip) {
                if let Some(p) = p2_opt {
                    if o1 & p.occupy != 0 { continue; }
                }
                let (do2, df2, ds2) = unpack(p2_opt);
                let o2 = o1 | do2;
                let f2 = f1 | df2;
                let s2 = s1 | ds2;

                for p3_opt in iter_or_skip(&placements[2], allow_skip) {
                    if let Some(p) = p3_opt {
                        if o2 & p.occupy != 0 { continue; }
                    }
                    let (do3, df3, ds3) = unpack(p3_opt);
                    let o3 = o2 | do3;
                    let f3 = f2 | df3;
                    let s3 = s2 | ds3;

                    for p4_opt in iter_or_skip(&placements[3], allow_skip) {
                        if let Some(p) = p4_opt {
                            if o3 & p.occupy != 0 { continue; }
                        }
                        let (do4, df4, ds4) = unpack(p4_opt);
                        let o4 = o3 | do4;
                        let front = f3 | df4;
                        let side = s3 | ds4;

                        local.combos_seen += 1;

                        // Strict ⇒ permissive, so on each empty placement we
                        // gate the strict check behind the permissive one:
                        //
                        //   - !perm_ok yet: test permissive first. If it fails,
                        //     strict also fails for this placement → skip with
                        //     1 call. If it passes, mark perm_ok and test strict
                        //     on the same placement (2 calls).
                        //   - perm_ok already set: we no longer need the
                        //     permissive answer, only strict. Test strict
                        //     directly (1 call). Break on success.
                        //
                        // For unstable 4-tuples (no empty arrangement balances)
                        // we exhaust the empty space at 1 call per placement
                        // instead of 2 — this is where the speedup lives.
                        let mut strict_ok = false;
                        let mut perm_ok = false;
                        'outer_empty: for p5_opt in iter_or_skip(&placements[4], allow_skip) {
                            if let Some(p) = p5_opt {
                                if o4 & p.occupy != 0 { continue; }
                            }
                            let do5 = p5_opt.map_or(0u64, |p| p.occupy);
                            let o5 = o4 | do5;
                            for p6_opt in iter_or_skip(&placements[5], allow_skip) {
                                if let Some(p) = p6_opt {
                                    if o5 & p.occupy != 0 { continue; }
                                }
                                // Stability gets only the actually-placed pieces.
                                let mut active: Vec<&Placement> = Vec::with_capacity(6);
                                for opt in [p1_opt, p2_opt, p3_opt, p4_opt, p5_opt, p6_opt] {
                                    if let Some(p) = opt { active.push(p); }
                                }

                                if !perm_ok {
                                    if !balance::check_stability_quick(&active, 0.0)
                                        .is_stable()
                                    {
                                        continue;
                                    }
                                    // if render_view(front) == "##.|#.#|###" && render_view(side) == "###|###|.#." {
                                    //     render::save_html(&active, "cube_views");
                                    // }
                                    perm_ok = true;
                                }
                                if perm_ok && balance::check_stability_quick(&active, STRICT_MARGIN)
                                    .is_stable()
                                {
                                    strict_ok = true;
                                    break 'outer_empty;
                                }
                            }
                        }

                        if perm_ok {
                            local.permissive_solutions += 1;
                            let entry = local.views.entry((front, side)).or_insert((0, 0));
                            entry.1 += 1;
                            if strict_ok {
                                local.strict_solutions += 1;
                                entry.0 += 1;
                            }
                        }
                    }
                }
            }

            let progress = done.fetch_add(1, Ordering::Relaxed) + 1;
            eprintln!(
                "outer #{:>2}/{}  combos={:>10}  strict={:>9}  perm={:>9}  views={:>6}  elapsed={:.1}s",
                progress,
                total_outer,
                local.combos_seen,
                local.strict_solutions,
                local.permissive_solutions,
                local.views.len(),
                started.elapsed().as_secs_f64(),
            );

            local
        })
        .reduce(LocalCounts::default, LocalCounts::merge);

    // ------------------------------------------------------------------
    //  Output: full table, sorted by strict count desc, then by marginal
    //  count desc (so the rare permissive-only puzzles bubble up after the
    //  strict ones). The marginal column is permissive − strict.
    // ------------------------------------------------------------------
    let mut sorted_views: Vec<((u16, u16), (u64, u64))> = summary.views.into_iter().collect();
    sorted_views.sort_by(|a, b| {
        let (a_strict, a_perm) = a.1;
        let (b_strict, b_perm) = b.1;
        let a_marg = a_perm - a_strict;
        let b_marg = b_perm - b_strict;
        b_strict
            .cmp(&a_strict)
            .then(b_marg.cmp(&a_marg))
            .then(a.0.cmp(&b.0))
    });

    let path = output_path.as_str();
    let file = File::create(path).expect("create output file");
    let mut w = BufWriter::new(file);
    writeln!(w, "# Stable solutions per (front, side) view of the four visible (non-empty) tiles.").unwrap();
    writeln!(w, "# A 'solution' is a distinct (triple, corner, double, single) 4-tuple for which").unwrap();
    writeln!(w, "# at least one placement of the two empty tiles makes the tower stand.").unwrap();
    writeln!(w, "#").unwrap();
    writeln!(w, "# STRICT   = solutions that stand with a safety margin (no tile balances on a").unwrap();
    writeln!(w, "#            support boundary). margin = {:.0e} cell-widths.", STRICT_MARGIN).unwrap();
    writeln!(w, "# MARGINAL = solutions that stand only when boundary balance is allowed").unwrap();
    writeln!(w, "#            (some tile sits exactly on the edge of its support polygon).").unwrap();
    writeln!(w, "# Total stable (permissive) = STRICT + MARGINAL.").unwrap();
    writeln!(w, "#").unwrap();
    writeln!(w, "# View grids: top row = z=2, rows separated by `|`, '#' = filled, '.' = empty.").unwrap();
    writeln!(w, "# Format:  STRICT   MARGINAL   FRONT          SIDE").unwrap();
    writeln!(w).unwrap();
    for ((front, side), (strict, perm)) in &sorted_views {
        let marginal = perm - strict;
        writeln!(
            w,
            "{:>8}   {:>8}   {}   {}",
            strict,
            marginal,
            render_view(*front),
            render_view(*side)
        )
        .unwrap();
    }
    w.flush().unwrap();

    let elapsed = started.elapsed().as_secs_f64();
    let marginal_total = summary.permissive_solutions - summary.strict_solutions;
    println!("non-overlapping 4-tuples (visible only): {}", summary.combos_seen);
    println!("strict solutions:                        {}", summary.strict_solutions);
    println!("marginal-only solutions:                 {}", marginal_total);
    println!("permissive solutions (strict + marg.):   {}", summary.permissive_solutions);
    println!("distinct (front, side) view pairs:       {}", sorted_views.len());
    println!("elapsed:                                 {:.2}s", elapsed);
    println!("table written to {}", path);
}
