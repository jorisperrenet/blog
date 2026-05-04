//! For every (front_card, side_card, front_orient, side_orient) 4-tuple
//! drawn from a deck file (default `cards.txt`; configurable via
//! `PROJECT_CUBE_CARDS=cards_original.txt` to match the blog's hardest-pairs
//! widget) — with the same card disallowed for both roles —
//! emit the stable / marginal / impossible counts by looking the resulting
//! (front, side) view pair up in `view_counts.txt`.
//!
//! - "Orientations" are the 4 rotations × 2 reflections of a 3×3 silhouette,
//!   deduplicated by 9-bit mask so a symmetric card doesn't show identical
//!   orientations multiple times.
//! - STABLE     = STRICT column from view_counts.txt (count of strictly
//!                stable 4-tuples of visible tiles with that silhouette).
//! - MARGINAL   = MARGINAL column from view_counts.txt (boundary balance).
//! - IMPOSSIBLE = 1 if the view pair has no entry in view_counts.txt at all
//!                (no stable tower can produce these silhouettes), else 0.
//!
//! Output goes to stdout, sorted by STABLE desc then MARGINAL desc.
//! Status lines go to stderr. Run with `cargo run --release --bin card_counts`.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
struct Card {
    id: u32,
    mask: u16,
}

/// Parse "row|row|row" (top row first; `#` filled, `.` empty) into a 9-bit
/// mask with bit `(a*3 + z)` set iff column `a`, row `z` is filled
/// (z = 0 = bottom). Same convention as `render_view` in `main.rs`.
fn parse_grid(s: &str) -> u16 {
    let rows: Vec<&str> = s.split('|').collect();
    assert_eq!(rows.len(), 3, "expected 3 rows in {:?}", s);
    let mut mask = 0u16;
    for (i, row) in rows.iter().enumerate() {
        let z = 2 - i; // first row in input = top = z=2
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

fn read_cards(path: &str) -> Vec<Card> {
    let file = File::open(path).unwrap_or_else(|e| panic!("open {}: {}", path, e));
    let mut cards = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.expect("read line");
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let id: u32 = parts
            .next()
            .expect("card id")
            .parse()
            .expect("card id is integer");
        let pattern = parts.next().expect("card pattern");
        cards.push(Card {
            id,
            mask: parse_grid(pattern),
        });
    }
    cards
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
        map.insert(
            (parse_grid(front), parse_grid(side)),
            (strict, marginal),
        );
    }
    map
}

/// Rotate a 3×3 mask 90° clockwise: cell (a, z) -> (z, 2-a).
fn rotate90(mask: u16) -> u16 {
    let mut new_mask = 0u16;
    for a in 0..3u16 {
        for z in 0..3u16 {
            if (mask >> (a * 3 + z)) & 1 == 1 {
                let new_a = z;
                let new_z = 2 - a;
                new_mask |= 1u16 << (new_a * 3 + new_z);
            }
        }
    }
    new_mask
}

/// Mirror left/right: cell (a, z) -> (2-a, z).
fn flip_h(mask: u16) -> u16 {
    let mut new_mask = 0u16;
    for a in 0..3u16 {
        for z in 0..3u16 {
            if (mask >> (a * 3 + z)) & 1 == 1 {
                let new_a = 2 - a;
                new_mask |= 1u16 << (new_a * 3 + z);
            }
        }
    }
    new_mask
}

/// All distinct D4 orientations of `mask` (4 rotations × {identity, flip_h}),
/// deduped by mask so symmetric cards don't show up multiple times.
fn orientations(mask: u16) -> Vec<u16> {
    let mut set = HashSet::new();
    let mut m = mask;
    for _ in 0..4 {
        set.insert(m);
        set.insert(flip_h(m));
        m = rotate90(m);
    }
    let mut v: Vec<u16> = set.into_iter().collect();
    v.sort();
    v
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

fn main() {
    // Cards file is configurable so the binary can be run against either the
    // printed deck (`cards_original.txt`, 15 cards — what the blog post's
    // hardest-pairs widget uses) or the extended scratch deck (`cards.txt`,
    // 19 cards including the experimental additions).
    let cards_path = std::env::var("PROJECT_CUBE_CARDS")
        .unwrap_or_else(|_| "cards.txt".to_string());
    eprintln!("reading cards from {}", cards_path);
    let cards = read_cards(&cards_path);
    eprintln!("loaded {} cards", cards.len());
    let view_counts = read_view_counts("view_counts.txt");
    eprintln!("loaded {} view-pair entries", view_counts.len());

    let card_orientations: Vec<Vec<u16>> =
        cards.iter().map(|c| orientations(c.mask)).collect();
    for (c, os) in cards.iter().zip(card_orientations.iter()) {
        eprintln!(
            "  card {:>2} ({}): {} distinct orientation(s)",
            c.id,
            render(c.mask),
            os.len()
        );
    }

    let n = cards.len();
    // (front_card_id, side_card_id, front_view_mask, side_view_mask,
    //  stable, marginal, impossible)
    let mut results: Vec<(u32, u32, u16, u16, u64, u64, u64)> = Vec::new();
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue; // same physical card cannot fill both roles
            }
            for &fr in &card_orientations[i] {
                for &sd in &card_orientations[j] {
                    let (stable, marginal, impossible) = match view_counts.get(&(fr, sd)) {
                        Some(&(s, m)) => (s, m, 0u64),
                        None => (0u64, 0u64, 1u64),
                    };
                    results.push((
                        cards[i].id,
                        cards[j].id,
                        fr,
                        sd,
                        stable,
                        marginal,
                        impossible,
                    ));
                }
            }
        }
    }

    results.sort_by(|a, b| {
        b.4.cmp(&a.4)            // STABLE desc
            .then(b.5.cmp(&a.5)) // MARGINAL desc
            .then(a.6.cmp(&b.6)) // IMPOSSIBLE asc (so 0s float to the top)
            .then(a.0.cmp(&b.0)) // then card / orient as deterministic tiebreaks
            .then(a.1.cmp(&b.1))
            .then(a.2.cmp(&b.2))
            .then(a.3.cmp(&b.3))
    });

    eprintln!("emitted {} 4-tuple rows", results.len());

    println!("# Per (front_card, side_card, front_orient, side_orient) row counts.");
    println!("# Same card cannot be both front and side. Orientations are deduped D4");
    println!("# transforms of each card's silhouette.");
    println!("# STABLE     = STRICT count from view_counts.txt for that (front, side) view");
    println!("# MARGINAL   = MARGINAL count from view_counts.txt for that view");
    println!("# IMPOSSIBLE = 1 if the view pair has no entry in view_counts.txt (no stable");
    println!("#              tower can produce these silhouettes), else 0");
    println!("#");
    println!("# Sorted by STABLE desc, MARGINAL desc.");
    println!("#");
    println!(
        "# {:>2} {:>2}  {:<11}  {:<11}  {:>8}  {:>8}  {:>10}",
        "FC", "SC", "FRONT_VIEW", "SIDE_VIEW", "STABLE", "MARGINAL", "IMPOSSIBLE"
    );
    for &(fc, sc, fr, sd, stable, marginal, impossible) in &results {
        println!(
            "  {:>2} {:>2}  {:<11}  {:<11}  {:>8}  {:>8}  {:>10}",
            fc,
            sc,
            render(fr),
            render(sd),
            stable,
            marginal,
            impossible
        );
    }
}
