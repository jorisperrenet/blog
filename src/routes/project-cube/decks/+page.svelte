<script>
  import { base } from '$app/paths';
</script>

<svelte:head><title>All decks — Project Cube</title></svelte:head>

<!-- Section 7 — All decks: every possible silhouette                -->
<!-- ============================================================== -->

<h2>All decks: every possible silhouette</h2>

<p>From the enumeration we got 72,994 distinct (front, side) pairs. We can do a nice analysis on those. Namely, which decks can be formed, so that every pair (with all rotations and reflections) of front and side views are constructable.</p>

<p>First, we construct the distinct silhouettes (under \(D_4\)). It turns out that there are 102 distinct silhouettes, of which only 12 are printed in the Project Cube deck. So pick any subset <em>S</em> of those 102 shapes: when does <em>S</em> form a valid deck — every pair of cards in <em>S</em>, in any rotation, having a strict solution? And what are the largest such subsets?</p>

<p>We can construct a compatibility graph from this, listing all views and drawing an edge between them if there is at least one strict solution between them (for every rotation). As a note, I did not check if a card can be used with a copy of itself. All solutions are on <a href="https://github.com/jorisperrenet/blog" target="_blank" rel="noopener noreferrer">GitHub</a> and this analysis can be added there.</p>

<h3>When not all pieces need to be used</h3>

<figure>
  <iframe class="iframe-graph" src={`${base}/project-cube/assets/compat_graph_subset.html`} loading="lazy" title="compatibility graph — pieces may be skipped"></iframe>
  <figcaption><strong>Subset-allowed regime</strong> (apparently the actual game's rules). 339 compatibility edges out of a possible 5,151. Sixty-three of the 102 silhouettes are isolated — they have no compatible partner at all.</figcaption>
</figure>

<h3>When all pieces need to be used</h3>

<figure>
  <iframe class="iframe-graph" src={`${base}/project-cube/assets/compat_graph_all.html`} loading="lazy" title="compatibility graph — all pieces required"></iframe>
  <figcaption><strong>All-pieces-required regime</strong> (a stricter version of the rules). The same 102 silhouettes, but only 285 compatibility edges remain.</figcaption>
</figure>

<p>The <a href="solver"><strong>silhouette pair solver</strong></a> uses these graphs' edges as its lookup table — every pair you can draw on the front and side grids is one entry in the subset-allowed graph above.</p>

<h3>The three largest cliques (subset-allowed regime)</h3>

<p>There are 71 maximal <a href="https://en.wikipedia.org/wiki/Clique_(graph_theory)" target="_blank" rel="noopener noreferrer">cliques</a> of size ≥ 2 in the subset-allowed graph. The three biggest all have <strong>15 nodes</strong>; no clique grows past that.</p>

<p>(Each clique block below also lists a <em>min STRICT</em> number. That's the smallest STRICT count for any pair of cards in the clique, in any orientation. It's the "weakest link": the rarest strict solution you'd ever face if you played with this entire clique as your deck. A min STRICT of 1 means somewhere in the clique there's a card pair with exactly one matching tower.)</p>

<div id="cliques-host">
  <!-- populated by JS from CLIQUES_DATA below -->
</div>

<h3>The all-pieces-required regime</h3>

<p>Recompute the same exercise against the stricter graph (the one that requires every tower to use all six pieces, second figure above). The graph has 79 maximal cliques of size ≥ 2 instead of 71, but they're <em>smaller</em> — the largest is now 13 nodes, the next two are 12. Three game-card shapes that were comfortably part of every top-3 subset-allowed clique drop out of the top cliques here: cards 8 (<span class="silh">###|#..|#..</span>), 10 (<span class="silh">#..|###|#..</span>), 12 (<span class="silh">.#.|###|.#.</span>) and 14 (<span class="silh">#..|##.|.##</span>) do not appear among the top three anymore.</p>

<div id="cliques-all-host">
  <!-- populated by JS from CLIQUES_ALL_DATA below -->
</div>

<!-- ============================================================== -->
