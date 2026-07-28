<script>
  import { base } from '$app/paths';
  import BlogHead from '$lib/BlogHead.svelte';
</script>

<BlogHead
  title="Project Cube: well-designed or not?"
  description="An analysis of the Project Cube puzzle game by Alain Rivollet: rules, deck composition, when 3×3×3 structures balance, the compatibility graph, and a live silhouette pair solver."
  path="/blog/project-cube/"
  published="2026-05-04"
/>

<h1>Project Cube: well-designed or not?</h1>

<p class="-mt-2 mb-5 text-[0.95em] text-gray-600 dark:text-gray-300">
  In a hurry? Skip to the <a href={`${base}/project-cube/solver/`}><strong>live silhouette pair solver</strong></a> and start drawing card pairs.
</p>

<!-- ============================================================== -->
<!-- Section 1 — Hook                                                -->
<!-- ============================================================== -->

<p>Project Cube is a small wooden puzzle game by Alain Rivollet. Two players race to build a 3-D figure whose front silhouette matches one card and whose side silhouette matches another. The deck has fifteen cards.</p>

<figure class="mx-auto mb-[0.6em] mt-[0.2em] w-full sm:float-right sm:ml-[1.2em] sm:w-[260px]">
  <iframe src={`${base}/project-cube/tower_view.html?example=easiest_game_pair`} loading="lazy" title="example complete tower with its two silhouettes" class="h-[260px]"></iframe>
  <figcaption class="text-[0.85em]">A complete tower for one card pair, with the two satisfied silhouettes floating behind it. Drag to rotate.</figcaption>
</figure>

<p>The natural question — the one that started this — is: <strong>does every pair of cards have a stable solution?</strong></p>

<p>My quest to solve it started with the obvious thought: <em>well, how many possible configurations of the blocks are there?</em> I picked Rust to implement this. After about thirty minutes I was stoked — the enumeration was done. The one thing I'd left as a <code>TODO</code> was the small matter of <em>when does something balance?</em> Little did I know that this was actually the difficult part — not the enumeration, but <em>given a 3-D structure, decide whether it is stable</em>. Once I'd worked that out, the rest fell into place: an analysis of the game, an answer to whether the deck could be extended with new cards, and a <a href={`${base}/project-cube/solver/`}>live solver</a> you can reference during gameplay for whichever pair you've drawn.</p>

<div class="clear-both"></div>

<div class="credit">
<strong>Project Cube</strong> is © Alain Rivollet, all rights reserved. This post is an analysis, not a substitute. The fun bit of the puzzle is the head-to-head race against someone else — sitting across from a person staring at the same two cards, both finding a particular puzzle way more difficult than the other. That experience is in the box, not on this page. Right — now that the obligatory disclaimer's out of the way, on with the maths.
</div>
