//! Static-stability check for stacked-block configurations on a 3×3×3 grid.
//!
//! The check is rigorous: it implements Heyman's safe-theorem / lower-bound
//! limit analysis for rigid blocks with no-tension contacts. Concretely, we
//! set up a feasibility LP that asks whether *any* assignment of non-negative
//! normal forces at every block-block (and block-ground) contact satisfies
//! force and moment balance for every block. Feasible ⇔ stable.
//!
//! No glue (contact forces ≥ 0), no horizontal loads (gravity only), and
//! axis-aligned flat contacts mean Coulomb friction is never the binding
//! constraint, so it is omitted. The check therefore returns `Stable` iff a
//! real-world tower of these blocks would stand under gravity (mathematically
//! marginal balance — COM exactly on a support edge — counts as stable).
//!
//! At each unit (1×1) face contact we expose 4 non-negative corner forces.
//! These span the full convex cone of admissible centers-of-pressure within
//! the face, capturing the moment freedom a real flat contact has. This is
//! the standard polyhedral contact model used in masonry limit analysis and
//! robotic grasping.

use std::collections::{HashMap, HashSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

use minilp::{ComparisonOp, OptimizationDirection, Problem, Variable};

/// One-shot guard so we only print the "minilp panicked" warning once per run.
static PANIC_WARNED: AtomicBool = AtomicBool::new(false);

/// Install (once, process-wide) a panic hook that swallows panics whose
/// `location().file()` is inside the `minilp` crate. Everything else still
/// goes to the previously-installed (typically default) hook. This avoids
/// the per-call `set_hook`/restore dance, which races under rayon and lets
/// the default hook print minilp panic banners on the threads that lose.
fn install_minilp_panic_filter() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if let Some(loc) = info.location() {
                if loc.file().contains("minilp") {
                    return;
                }
            }
            prev(info);
        }));
    });
}

/// Cell coordinate in the 3×3×3 grid; each component must be 0..=2.
pub type CellPos = (u8, u8, u8);

/// Result of a stability check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stability {
    /// At least one valid normal-force distribution exists. Tower stands.
    Stable,
    /// No valid normal-force distribution exists. Some block tips.
    Unstable,
    /// A block has no supported bottom cell (no ground contact, no block under it).
    Floating,
    /// Two blocks share a cell, or a coordinate is outside 0..=2.
    Invalid,
}

impl Stability {
    #[inline]
    pub fn is_stable(self) -> bool {
        matches!(self, Stability::Stable)
    }
}

/// Pre-baked metadata for one placement of one tile. Built once and reused
/// across the enumeration loop so the hot path doesn't allocate.
#[derive(Clone, Debug)]
pub struct Placement {
    /// Bitmask of cells occupied by this placement (bit = x*9 + 3y + z).
    pub occupy: u64,
    pub cells: Vec<CellPos>,
    pub mass: f64,
    /// Mass-weighted x-coordinate of the cell-uniform-density COM.
    pub com_x: f64,
    pub com_y: f64,
    /// Bitmask of the cells immediately below each bottom-of-column cell of
    /// this tile (only those at z > 0). Tile is supported by another tile
    /// iff this mask intersects that tile's `occupy`.
    pub support_below_mask: u64,
    /// True iff at least one bottom-of-column cell of this tile is at z = 0.
    pub has_ground_support: bool,
    /// Front-view (xz) silhouette of this placement. Bit (x*3 + z) is set iff
    /// this placement occupies any cell at column x, height z (any y).
    pub front_mask: u16,
    /// Side-view (yz) silhouette of this placement. Bit (y*3 + z) is set iff
    /// this placement occupies any cell at column y, height z (any x).
    pub side_mask: u16,
}

impl Placement {
    pub fn new(cells: Vec<CellPos>, mass: f64) -> Self {
        let occupy = cells.iter().fold(0u64, |acc, &(x, y, z)| {
            acc | (1u64 << (x as u32 * 9 + 3 * y as u32 + z as u32))
        });
        let mut support_below_mask = 0u64;
        let mut has_ground_support = false;
        for &(x, y, z) in &cells {
            let is_bottom = if z == 0 {
                true
            } else {
                let below_bit = x as u32 * 9 + 3 * y as u32 + (z as u32 - 1);
                (occupy & (1u64 << below_bit)) == 0
            };
            if is_bottom {
                if z == 0 {
                    has_ground_support = true;
                } else {
                    let below_bit = x as u32 * 9 + 3 * y as u32 + (z as u32 - 1);
                    support_below_mask |= 1u64 << below_bit;
                }
            }
        }
        let n = cells.len() as f64;
        let com_x = cells.iter().map(|c| c.0 as f64).sum::<f64>() / n;
        let com_y = cells.iter().map(|c| c.1 as f64).sum::<f64>() / n;
        let mut front_mask = 0u16;
        let mut side_mask = 0u16;
        for &(x, y, z) in &cells {
            front_mask |= 1u16 << (x * 3 + z);
            side_mask |= 1u16 << (y * 3 + z);
        }
        Placement {
            occupy,
            cells,
            mass,
            com_x,
            com_y,
            support_below_mask,
            has_ground_support,
            front_mask,
            side_mask,
        }
    }
}

struct Contact {
    upper: usize,
    lower: Option<usize>, // None = ground
    x: u8,
    y: u8,
}

/// Determine whether the configuration of blocks is statically stable under gravity.
///
/// Each block is `(cells, mass_in_grams)`. All cells must lie within 0..=2
/// on each axis and no two blocks may share a cell.
///
/// `margin` is a non-negative offset (in cell-width units) by which each unit
/// contact face is eroded inward on every edge before the LP is built. Setting
/// `margin = 0.0` reproduces the classical Heyman lower-bound check (a tower
/// balancing on the very edge of its support counts as `Stable`). Setting
/// `margin > 0.0` enforces a safety margin: the same tower would now report
/// `Unstable` because no normal-force distribution within the eroded contacts
/// can hold up the load. Same theorem, tighter input polygon.
pub fn check_stability(blocks: &[(&[CellPos], f64)], margin: f64) -> Stability {
    // ---- 1. Geometric validation -----------------------------------------
    let mut occupied: HashMap<CellPos, usize> = HashMap::new();
    for (i, (cells, _)) in blocks.iter().enumerate() {
        for &c in *cells {
            if c.0 > 2 || c.1 > 2 || c.2 > 2 {
                return Stability::Invalid;
            }
            if occupied.insert(c, i).is_some() {
                return Stability::Invalid;
            }
        }
    }

    // ---- 2. Enumerate contacts; verify every block has ≥1 support --------
    let mut contacts: Vec<Contact> = Vec::new();
    for (i, (cells, _)) in blocks.iter().enumerate() {
        let cell_set: HashSet<CellPos> = cells.iter().copied().collect();
        let mut supported = false;
        for &(x, y, z) in *cells {
            // Skip cells that have a same-block cell directly beneath — those
            // are interior to the block, not its bottom face in column (x,y).
            if z > 0 && cell_set.contains(&(x, y, z - 1)) {
                continue;
            }
            if z == 0 {
                contacts.push(Contact { upper: i, lower: None, x, y });
                supported = true;
            } else if let Some(&j) = occupied.get(&(x, y, z - 1)) {
                contacts.push(Contact { upper: i, lower: Some(j), x, y });
                supported = true;
            }
            // else: this bottom cell is in mid-air; no contact added.
        }
        if !supported {
            return Stability::Floating;
        }
    }

    // ---- 3. Build the feasibility LP -------------------------------------
    let mut problem = Problem::new(OptimizationDirection::Minimize);

    // 4 corner-force variables per contact, each ≥ 0.
    let vars: Vec<[Variable; 4]> = contacts
        .iter()
        .map(|_| {
            [
                problem.add_var(0.0, (0.0, f64::INFINITY)),
                problem.add_var(0.0, (0.0, f64::INFINITY)),
                problem.add_var(0.0, (0.0, f64::INFINITY)),
                problem.add_var(0.0, (0.0, f64::INFINITY)),
            ]
        })
        .collect();

    // Per block: 1 vertical force balance + 2 moment balances.
    for (b_idx, (cells, mass)) in blocks.iter().enumerate() {
        let n = cells.len() as f64;
        let com_x: f64 = cells.iter().map(|c| c.0 as f64).sum::<f64>() / n;
        let com_y: f64 = cells.iter().map(|c| c.1 as f64).sum::<f64>() / n;
        let w = *mass;

        let mut force = Vec::<(Variable, f64)>::new();
        let mut mom_y = Vec::<(Variable, f64)>::new(); // moment about y-axis ↔ F·x
        let mut mom_x = Vec::<(Variable, f64)>::new(); // moment about x-axis ↔ F·y

        for (c_idx, contact) in contacts.iter().enumerate() {
            // +1 if the contact pushes B up (B is above the contact);
            // -1 if B is below the contact and feels the reaction pressing down.
            let sign = if contact.upper == b_idx {
                1.0
            } else if contact.lower == Some(b_idx) {
                -1.0
            } else {
                continue;
            };

            let cx = contact.x as f64;
            let cy = contact.y as f64;
            // Erode the unit contact face inward by `margin` on each edge.
            // For margin = 0 this is the original 1×1 cell footprint.
            let off = 0.5 - margin;
            let corners = [
                (cx - off, cy - off),
                (cx - off, cy + off),
                (cx + off, cy - off),
                (cx + off, cy + off),
            ];

            for k in 0..4 {
                let v = vars[c_idx][k];
                let (px, py) = corners[k];
                force.push((v, sign));
                mom_y.push((v, sign * px));
                mom_x.push((v, sign * py));
            }
        }

        problem.add_constraint(force, ComparisonOp::Eq, w);
        problem.add_constraint(mom_y, ComparisonOp::Eq, w * com_x);
        problem.add_constraint(mom_x, ComparisonOp::Eq, w * com_y);
    }

    // ---- 4. Solve ---------------------------------------------------------
    // minilp 0.2.x can panic with `SingularMatrix` on degenerate LPs (e.g.
    // when `margin` lands several contact corners on the same support edge).
    // The process-wide panic filter (installed once below) silences the
    // banner; `catch_unwind` localises the unwind so this thread continues.
    // We treat a panicking solve as "solver gave up" — not demonstrably
    // stable, so report Unstable. Conservative: any config we still count
    // as Stable is genuinely LP-feasible.
    install_minilp_panic_filter();
    let result = catch_unwind(AssertUnwindSafe(|| problem.solve()));
    match result {
        Ok(Ok(_)) => Stability::Stable,
        Ok(Err(_)) => Stability::Unstable,
        Err(_) => {
            if !PANIC_WARNED.swap(true, Ordering::Relaxed) {
                eprintln!(
                    "warning: minilp panicked (likely SingularMatrix on a degenerate LP); \
                     treating as Unstable. Further occurrences will be suppressed."
                );
            }
            Stability::Unstable
        }
    }
}

// ===========================================================================
//  Fast path: bitmask floating check + conservative tipping check + LP only
//  for true bridge cases. Used by the enumeration hot loop.
// ===========================================================================

/// Stability check for a known-valid configuration, in this order:
///
/// 1. **Floating** check: pure bitmask, rejects most bad configs in ns.
/// 2. **Conservative tipping** check: for each tile, the COM of "tile + every
///    tile transitively stacked above it" must project within the convex hull
///    of that tile's actually-supported bottom cells (eroded by `margin` on
///    each edge). Sufficient for stability.
/// 3. **Bridge** check: if the conservative check fails but no tile rests on
///    multiple distinct supports, the conservative answer is exact and we
///    return `Unstable` immediately. Otherwise, fall through to the LP.
/// 4. **LP** fallback: the rigorous check from `check_stability`, called with
///    the same `margin` so the eroded support polygons are used there too.
///
/// `margin` is the same eroded-support-polygon parameter as in
/// `check_stability`; pass `0.0` for the classical (boundary-counts-as-stable)
/// answer and a small positive number to require a safety margin.
///
/// Caller MUST guarantee:
/// - No two placements share a cell.
/// - All cells are within 0..=2 on each axis.
pub fn check_stability_quick(placements: &[&Placement], margin: f64) -> Stability {
    let n = placements.len();
    let total_occupy: u64 = placements.iter().map(|p| p.occupy).fold(0, |a, b| a | b);

    // 1. Floating: every tile must have ground or a non-self tile beneath it.
    for p in placements {
        if !p.has_ground_support && (total_occupy & p.support_below_mask) == 0 {
            return Stability::Floating;
        }
    }

    // 2. Build the "above" graph: above[i] = bitmask of tiles directly resting on i.
    //    Tile j is directly above tile i iff some bottom-of-j cell sits on top of i.
    let mut above = [0u32; 8]; // n ≤ 6 in practice; cap at 8 for safety
    let mut support_count = [0u32; 8]; // # of distinct supports for each tile
    for j in 0..n {
        if placements[j].has_ground_support {
            support_count[j] += 1;
        }
        let supp = placements[j].support_below_mask;
        for i in 0..n {
            if i != j && (supp & placements[i].occupy) != 0 {
                above[i] |= 1u32 << j;
                support_count[j] += 1;
            }
        }
    }

    // 3. Conservative per-tile tipping check.
    let mut conservative_pass = true;
    for i in 0..n {
        // T(i) = i + transitive descendants in `above`.
        let mut t: u32 = 1u32 << i;
        loop {
            let mut new_t = t;
            let mut bits = t;
            while bits != 0 {
                let j = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                new_t |= above[j];
            }
            if new_t == t {
                break;
            }
            t = new_t;
        }

        // COM of T(i).
        let mut total_mass = 0.0f64;
        let mut sx = 0.0f64;
        let mut sy = 0.0f64;
        let mut bits = t;
        while bits != 0 {
            let j = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            let p = placements[j];
            sx += p.com_x * p.mass;
            sy += p.com_y * p.mass;
            total_mass += p.mass;
        }
        let load_com_x = sx / total_mass;
        let load_com_y = sy / total_mass;

        // Supported bottom cells of i (those with ground or another tile beneath).
        let p_i = placements[i];
        let mut supported: [(u8, u8); 9] = [(0, 0); 9];
        let mut sup_n: usize = 0;
        for &(x, y, z) in &p_i.cells {
            let is_bottom = if z == 0 {
                true
            } else {
                let bb = x as u32 * 9 + 3 * y as u32 + (z as u32 - 1);
                (p_i.occupy & (1u64 << bb)) == 0
            };
            if !is_bottom {
                continue;
            }
            let supported_here = if z == 0 {
                true
            } else {
                let bb = x as u32 * 9 + 3 * y as u32 + (z as u32 - 1);
                (total_occupy & (1u64 << bb)) != 0
            };
            if supported_here {
                supported[sup_n] = (x, y);
                sup_n += 1;
            }
        }

        if !point_in_polyomino_hull(load_com_x, load_com_y, &supported[..sup_n], margin) {
            conservative_pass = false;
            break;
        }
    }

    if conservative_pass {
        return Stability::Stable;
    }

    // 4. Bridge check. If no tile has more than one distinct support source,
    //    the conservative rule is exact and the configuration definitely tips.
    let any_bridge = (0..n).any(|j| support_count[j] > 1);
    if !any_bridge {
        return Stability::Unstable;
    }

    // 5. LP fallback.
    let blocks: Vec<(&[CellPos], f64)> = placements
        .iter()
        .map(|p| (p.cells.as_slice(), p.mass))
        .collect();
    check_stability(&blocks, margin)
}

/// True iff (px, py) lies inside (or on the boundary of) the convex hull of
/// the unit cell extents listed in `cells`, each eroded inward by `margin`.
///
/// `margin = 0.0` reproduces the classical check (boundary counts as inside).
/// `margin > 0.0` requires the point to sit at least `margin` cell-widths
/// inside the hull on every edge.
fn point_in_polyomino_hull(px: f64, py: f64, cells: &[(u8, u8)], margin: f64) -> bool {
    const EPS: f64 = 1e-9;
    let n = cells.len();
    if n == 0 {
        return false;
    }
    let off = 0.5 - margin;
    if off <= 0.0 {
        return false; // support eroded to nothing
    }
    if n == 1 {
        let (cx, cy) = cells[0];
        let cx = cx as f64;
        let cy = cy as f64;
        return px >= cx - off - EPS
            && px <= cx + off + EPS
            && py >= cy - off - EPS
            && py <= cy + off + EPS;
    }

    // Generate eroded cell-corner points (4 per cell, up to 36 total) and
    // compute the convex hull, then test the point against it.
    let mut corners: [(f64, f64); 36] = [(0.0, 0.0); 36];
    let mut nc = 0;
    for &(x, y) in cells {
        let cx = x as f64;
        let cy = y as f64;
        corners[nc] = (cx - off, cy - off);
        corners[nc + 1] = (cx + off, cy - off);
        corners[nc + 2] = (cx + off, cy + off);
        corners[nc + 3] = (cx - off, cy + off);
        nc += 4;
    }
    let mut hull: [(f64, f64); 36] = [(0.0, 0.0); 36];
    let hn = convex_hull_into(&mut corners[..nc], &mut hull);
    point_in_ccw_polygon(px, py, &hull[..hn])
}

/// Andrew's monotone-chain hull. Sorts `points` in place; writes the hull (in
/// CCW order) into `out` and returns its length. `out` must have capacity ≥ n.
fn convex_hull_into(points: &mut [(f64, f64)], out: &mut [(f64, f64)]) -> usize {
    let n = points.len();
    points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then(a.1.partial_cmp(&b.1).unwrap()));
    if n <= 1 {
        for i in 0..n {
            out[i] = points[i];
        }
        return n;
    }

    #[inline]
    fn cross(o: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
        (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
    }

    let mut k = 0usize; // hull length so far
    // Lower hull.
    for &p in points.iter() {
        while k >= 2 && cross(out[k - 2], out[k - 1], p) <= 0.0 {
            k -= 1;
        }
        out[k] = p;
        k += 1;
    }
    let lower_end = k + 1;
    // Upper hull.
    for &p in points.iter().rev().skip(1) {
        while k >= lower_end && cross(out[k - 2], out[k - 1], p) <= 0.0 {
            k -= 1;
        }
        out[k] = p;
        k += 1;
    }
    k - 1 // drop the last point (it's the same as the first)
}

fn point_in_ccw_polygon(px: f64, py: f64, hull: &[(f64, f64)]) -> bool {
    const EPS: f64 = 1e-9;
    let n = hull.len();
    if n == 0 {
        return false;
    }
    if n == 1 {
        return (px - hull[0].0).abs() < EPS && (py - hull[0].1).abs() < EPS;
    }
    if n == 2 {
        // Treat the 2-vertex degenerate hull as a line segment; accept points
        // on the segment within tolerance.
        let (a, b) = (hull[0], hull[1]);
        let cross = (b.0 - a.0) * (py - a.1) - (b.1 - a.1) * (px - a.0);
        if cross.abs() > EPS {
            return false;
        }
        let dot = (px - a.0) * (b.0 - a.0) + (py - a.1) * (b.1 - a.1);
        let len2 = (b.0 - a.0).powi(2) + (b.1 - a.1).powi(2);
        return dot >= -EPS && dot <= len2 + EPS;
    }
    for i in 0..n {
        let a = hull[i];
        let b = hull[(i + 1) % n];
        let cross = (b.0 - a.0) * (py - a.1) - (b.1 - a.1) * (px - a.0);
        if cross < -EPS {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(blocks: &[(Vec<CellPos>, f64)]) -> Stability {
        run_with_margin(blocks, 0.0)
    }

    fn run_with_margin(blocks: &[(Vec<CellPos>, f64)], margin: f64) -> Stability {
        let refs: Vec<(&[CellPos], f64)> =
            blocks.iter().map(|(c, m)| (c.as_slice(), *m)).collect();
        check_stability(&refs, margin)
    }

    #[test]
    fn single_cube_on_ground_is_stable() {
        assert_eq!(run(&[(vec![(1, 1, 0)], 40.0)]), Stability::Stable);
    }

    #[test]
    fn vertical_column_balances_on_one_cell() {
        assert_eq!(
            run(&[(vec![(1, 1, 0), (1, 1, 1), (1, 1, 2)], 120.0)]),
            Stability::Stable
        );
    }

    #[test]
    fn floating_block_detected() {
        assert_eq!(run(&[(vec![(1, 1, 2)], 40.0)]), Stability::Floating);
    }

    #[test]
    fn l_block_balances_on_one_foot() {
        // L-tromino standing in the xz-plane on a single foot cell.
        // Foot at (0,0,0); corner above at (0,0,1); arm at (1,0,1).
        // COM_x = 1/3 → inside the 1×1 footprint (margin 1/6). Stable.
        assert_eq!(
            run(&[(vec![(0, 0, 0), (0, 0, 1), (1, 0, 1)], 120.0)]),
            Stability::Stable
        );
    }

    #[test]
    fn l_block_flat_on_ground_is_stable() {
        // L-tromino lying flat in the xy-plane on the ground.
        // All three cells touch the ground → trivially stable.
        assert_eq!(
            run(&[(vec![(0, 0, 0), (1, 0, 0), (0, 1, 0)], 120.0)]),
            Stability::Stable
        );
    }

    #[test]
    fn l_block_standing_on_two_cells_is_stable() {
        // L-tromino standing in the xz-plane with both arm-ends on the ground.
        // (0,0,0) and (1,0,0) on ground; (0,0,1) on top of the first cell.
        assert_eq!(
            run(&[(vec![(0, 0, 0), (1, 0, 0), (0, 0, 1)], 120.0)]),
            Stability::Stable
        );
    }

    #[test]
    fn l_block_flat_on_single_base_balances() {
        // 1×1×1 base; flat L on top with its CORNER cell over the base and
        // both arms cantilevered. COM_xy = (1/3, 1/3) from the corner cell
        // centre → inside the 1×1 support footprint (margin 1/6 in each dir).
        assert_eq!(
            run(&[
                (vec![(0, 0, 0)], 40.0),
                (vec![(0, 0, 1), (1, 0, 1), (0, 1, 1)], 120.0),
            ]),
            Stability::Stable
        );
    }

    #[test]
    fn l_block_cantilevered_off_arm_tip_topples() {
        // Same flat L but the support sits under an ARM cell, not the corner.
        // L cells: (0,0,1) arm tip, (0,1,1) corner, (1,1,1) other arm tip.
        // COM_xy = (1/3, 2/3). Support cell at (0,0) → footprint y ∈ [-0.5, 0.5].
        // COM_y = 2/3 is outside the footprint → topples.
        assert_eq!(
            run(&[
                (vec![(0, 0, 0)], 40.0),
                (vec![(0, 0, 1), (0, 1, 1), (1, 1, 1)], 120.0),
            ]),
            Stability::Unstable
        );
    }

    #[test]
    fn cantilevered_long_arm_topples() {
        // 1×1×1 base + 1×1×3 horizontal on top, supported only at one end.
        // COM of top block at x=1, support at x=0 → topples.
        assert_eq!(
            run(&[
                (vec![(0, 0, 0)], 40.0),
                (vec![(0, 0, 1), (1, 0, 1), (2, 0, 1)], 120.0),
            ]),
            Stability::Unstable
        );
    }

    #[test]
    fn symmetric_bridge_balances() {
        // Two pillars at (0,0,0) and (2,0,0); 1×1×3 spans at z=1 with the
        // middle column (1,0,0) empty — true bridge. The naive per-block COM
        // rule would falsely reject this; the LP correctly accepts it.
        assert_eq!(
            run(&[
                (vec![(0, 0, 0)], 40.0),
                (vec![(2, 0, 0)], 40.0),
                (vec![(0, 0, 1), (1, 0, 1), (2, 0, 1)], 120.0),
            ]),
            Stability::Stable
        );
    }

    #[test]
    fn marginal_balance_on_edge() {
        // Top block's COM exactly on the edge of the support footprint.
        // At margin=0 the boundary counts as inside → Stable.
        // At any margin>0 the eroded footprint excludes the boundary → Unstable.
        let bs = vec![
            (vec![(1, 1, 0)], 40.0),
            (vec![(0, 1, 1), (1, 1, 1)], 80.0),
        ];
        assert_eq!(run_with_margin(&bs, 0.0), Stability::Stable);
        assert_eq!(run_with_margin(&bs, 1e-6), Stability::Unstable);
    }

    #[test]
    fn overlap_invalid() {
        assert_eq!(
            run(&[(vec![(0, 0, 0)], 40.0), (vec![(0, 0, 0)], 40.0)]),
            Stability::Invalid
        );
    }

    #[test]
    fn out_of_bounds_invalid() {
        assert_eq!(run(&[(vec![(3, 0, 0)], 40.0)]), Stability::Invalid);
    }

    // -------- check_stability_quick agreement with check_stability ---------
    fn run_quick(blocks: &[(Vec<CellPos>, f64)]) -> Stability {
        run_quick_with_margin(blocks, 0.0)
    }

    fn run_quick_with_margin(blocks: &[(Vec<CellPos>, f64)], margin: f64) -> Stability {
        let pls: Vec<Placement> =
            blocks.iter().map(|(c, m)| Placement::new(c.clone(), *m)).collect();
        let refs: Vec<&Placement> = pls.iter().collect();
        check_stability_quick(&refs, margin)
    }

    #[test]
    fn quick_matches_lp_on_basic_cases() {
        // single cube
        assert_eq!(run_quick(&[(vec![(1, 1, 0)], 40.0)]), Stability::Stable);
        // floating
        assert_eq!(run_quick(&[(vec![(1, 1, 2)], 40.0)]), Stability::Floating);
        // L on one foot — conservative passes (no bridge), fast path stable
        assert_eq!(
            run_quick(&[(vec![(0, 0, 0), (0, 0, 1), (1, 0, 1)], 120.0)]),
            Stability::Stable
        );
        // cantilever that topples — no bridge, fast path returns Unstable
        assert_eq!(
            run_quick(&[
                (vec![(0, 0, 0)], 40.0),
                (vec![(0, 0, 1), (1, 0, 1), (2, 0, 1)], 120.0),
            ]),
            Stability::Unstable
        );
    }

    #[test]
    fn quick_falls_through_to_lp_for_bridges() {
        // Two pillars + 1×1×3 spanning the gap. The 1×1×3 has TWO distinct
        // supports (pillar A and pillar B), so the conservative check rejects
        // it (load can't all go through one pillar) but the LP accepts it.
        // The quick check must therefore fall through to the LP and return
        // Stable.
        assert_eq!(
            run_quick(&[
                (vec![(0, 0, 0)], 40.0),
                (vec![(2, 0, 0)], 40.0),
                (vec![(0, 0, 1), (1, 0, 1), (2, 0, 1)], 120.0),
            ]),
            Stability::Stable
        );
    }

    #[test]
    fn pyramid_is_stable() {
        // 4 cells on the ground + 1 cell on top, centered.
        assert_eq!(
            run(&[
                (vec![(0, 0, 0)], 40.0),
                (vec![(1, 0, 0)], 40.0),
                (vec![(0, 1, 0)], 40.0),
                (vec![(1, 1, 0)], 40.0),
                // 1×1 sitting on the (0..=1, 0..=1) cluster — fully inside support hull.
                // Actually a 1×1×1 must sit in one cell, e.g. (0,0,1):
                (vec![(0, 0, 1)], 40.0),
            ]),
            Stability::Stable
        );
    }

    // ---- "..x / xxx / ..x" tower scenarios -------------------------------
    //
    // Geometry (y = 0 plane):
    //
    //   z=2:  . . x        only (2,0,2)
    //   z=1:  x x x        (0,0,1) (1,0,1) (2,0,1)
    //   z=0:  . . x        only (2,0,0)
    //
    // Single ground footprint at cell (2,0)  →  x ∈ [1.5, 2.5].
    // With every cell at full density (40 g): COM_x = (2+0+1+2+2)/5 = 1.4
    // → outside the footprint, so no full-density tiling can stand.

    #[test]
    fn cross_tower_with_horizontal_1x1x3_topples() {
        // 1×1×3 horizontal at z=1 + cubes at (2,0,0) and (2,0,2).
        // Tile COM_x = 1, plus the +x-loaded cube on top → T(tile) COM_x ≈ 1.25.
        // Cube at (2,0,0) is the only ground contact; its load = own weight + tile + top cube.
        // The conservative per-block check would already flag this; LP confirms.
        assert_eq!(
            run(&[
                (vec![(2, 0, 0)], 40.0),
                (vec![(0, 0, 1), (1, 0, 1), (2, 0, 1)], 120.0),
                (vec![(2, 0, 2)], 40.0),
            ]),
            Stability::Unstable
        );
    }

    #[test]
    fn cross_tower_with_xxx_dot_dot_x_tetromino_topples() {
        // 4-cell J-tetromino covers the middle row + bottom-right:
        //   (0,0,1) (1,0,1) (2,0,1) (2,0,0)
        // Plus a 1×1×1 cube at the top-right (2,0,2).
        // Tile COM_x = (0+1+2+2)/4 = 1.25. Only ground contact at (2,0).
        // Even with cube loading the right side, T(tile) COM_x = 1.4 < 1.5.
        assert_eq!(
            run(&[
                (vec![(0, 0, 1), (1, 0, 1), (2, 0, 1), (2, 0, 0)], 160.0),
                (vec![(2, 0, 2)], 40.0),
            ]),
            Stability::Unstable
        );
    }

    #[test]
    fn cross_tower_with_dot_dot_x_xxx_tetromino_topples() {
        // 4-cell L-tetromino covers the middle row + top-right:
        //   (0,0,1) (1,0,1) (2,0,1) (2,0,2)
        // Plus a 1×1×1 cube at the bottom-right (2,0,0).
        // Tile COM_x = 1.25. Tile rests on the cube only at (2,0) → footprint x ∈ [1.5, 2.5].
        assert_eq!(
            run(&[
                (vec![(2, 0, 0)], 40.0),
                (vec![(0, 0, 1), (1, 0, 1), (2, 0, 1), (2, 0, 2)], 160.0),
            ]),
            Stability::Unstable
        );
    }

    #[test]
    fn cross_tower_floats_with_disconnected_horizontal_pair() {
        // 1×1×3 vertical at x=2 + 1×1×2 horizontal at z=1 covering (0,0,1) and (1,0,1).
        // The 2-cell horizontal has no support beneath — it floats.
        assert_eq!(
            run(&[
                (vec![(2, 0, 0), (2, 0, 1), (2, 0, 2)], 120.0),
                (vec![(0, 0, 1), (1, 0, 1)], 80.0),
            ]),
            Stability::Floating
        );
    }

    #[test]
    fn one_cell_overhang_is_marginal_stable() {
        // 1×1×1 base at (1,1,0); 1×1×2 horizontal on top with one cell over the
        // base and one cell overhanging. COM_x of top block = 1.5, support
        // footprint x ∈ [0.5, 1.5] → COM exactly on the +x edge. The user's
        // canonical "barely balancing" case.
        let bs = vec![
            (vec![(1, 1, 0)], 40.0),
            (vec![(1, 1, 1), (2, 1, 1)], 80.0),
        ];
        assert_eq!(run_with_margin(&bs, 0.0), Stability::Stable);
        assert_eq!(run_with_margin(&bs, 1e-6), Stability::Unstable);
    }

    #[test]
    fn l_block_one_foot_has_safety_margin() {
        // L-tromino on a single foot: COM_x = 1/3, support cell (0,0) extent
        // x ∈ [-0.5, 0.5]. Distance from boundary = 1/6 ≈ 0.167.
        // → Stable up to margin ~0.166; unstable beyond.
        let bs = vec![(vec![(0, 0, 0), (0, 0, 1), (1, 0, 1)], 120.0)];
        assert_eq!(run_with_margin(&bs, 0.10), Stability::Stable);
        assert_eq!(run_with_margin(&bs, 0.20), Stability::Unstable);
    }

    #[test]
    fn quick_path_respects_margin() {
        // Same canonical 1×1×2-on-1×1×1 case via the fast path.
        let bs = vec![
            (vec![(1, 1, 0)], 40.0),
            (vec![(1, 1, 1), (2, 1, 1)], 80.0),
        ];
        assert_eq!(run_quick_with_margin(&bs, 0.0), Stability::Stable);
        assert_eq!(run_quick_with_margin(&bs, 1e-6), Stability::Unstable);
    }

    #[test]
    fn quick_bridge_respects_margin() {
        // Symmetric bridge (load at the geometric centre of two pillars) is
        // robustly stable: the LP places half the load on each pillar's inner
        // corner, and that distribution survives any reasonable margin.
        let bs = vec![
            (vec![(0, 0, 0)], 40.0),
            (vec![(2, 0, 0)], 40.0),
            (vec![(0, 0, 1), (1, 0, 1), (2, 0, 1)], 120.0),
        ];
        assert_eq!(run_quick_with_margin(&bs, 0.0), Stability::Stable);
        assert_eq!(run_quick_with_margin(&bs, 0.10), Stability::Stable);
    }
}
