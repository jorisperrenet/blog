<script>
  import { base } from '$app/paths';
</script>

<svelte:head><title>The cards — Project Cube</title></svelte:head>

<!-- Section 4 — The cards                                           -->
<!-- ============================================================== -->

<h2>The cards</h2>

<p>With the question of balance now settled, the next step is to look at the cards themselves. There are fifteen of them.</p>

<figure>
  <iframe class="iframe-cards" src={`${base}/project-cube/assets/cards.html`} loading="lazy" title="all 15 cards"></iframe>
  <figcaption>The fifteen printed cards. Each silhouette is a 3×3 grid; filled squares mean "must be a solid (non-hollow) block here when viewed from this side."</figcaption>
</figure>

<p>Now a small surprise. The instructions say rotation is part of the puzzle — you can put a card into the holder any of the four 90° ways and it counts as the same card. So when counting <em>distinct</em> silhouettes, every rotation should collapse to the same one. There are four rotations (0°, 90°, 180°, 270°) — under that grouping, the deck has fifteen distinct shapes, one per card.</p>

<p>But now we have forgotten something, namely reflections. Cards can also be mirrored (the back is the same as the front but reflected, for the player sitting next to you to build the same tower). Not that mirroring does anything to the solutions: as all pieces can be mirrored, if a figure can be built, its mirror also exists.</p>

<p>So, we can combine one of <strong>eight</strong> symmetries: the four rotations and their reflections. Mathematicians call this the <a href="https://en.wikipedia.org/wiki/Dihedral_group" target="_blank" rel="noopener noreferrer">dihedral group</a> \(D_4\), the symmetry group of the square. Under that grouping, the deck has only <strong>twelve</strong> shapes, because three pairs of cards are mirror images of each other:</p>

<ul>
<li>Card 1 (<span class="silh">###|##.|.#.</span>) is the mirror of card 3 (<span class="silh">##.|###|#..</span>)</li>
<li>Card 5 (<span class="silh">#..|###|.##</span>) is the mirror of card 7 (<span class="silh">##.|##.|.##</span>)</li>
<li>Card 9 (<span class="silh">.##|##.|.#.</span>) is the mirror of card 11 (<span class="silh">##.|.##|.#.</span>)</li>
</ul>

<p>Therefore, we say that the original deck has 12 distinct cards (under \(D_4\)) and not 15 distinct ones. From here on, when this post says "distinct cards," it means twelve.</p>

<!-- ============================================================== -->
