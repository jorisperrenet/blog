<script>
  import Silhouette from '$lib/Silhouette.svelte';
  import Math from '$lib/Math.svelte';
</script>

<svelte:head><title>Conclusion — Project Cube</title></svelte:head>

<h2>So — well-designed or not?</h2>

<p>To answer the opening question: <strong>yes</strong>. Every pair of cards in the printed deck has a stable solution, in any rotation, under the actual game's rule that piece-skipping is allowed. The deck is a clique in the compatibility graph, with no missing edges. The 12 <Math expr="D_4" />-canonical shapes were not picked at random — they form a genuine subset of the 102 possible silhouettes such that every pair plays.</p>

<p>The deck could be extended: <Silhouette text="##.|###|###" /> and <Silhouette text="###|#.#|###" /> can be added without breaking anything. Several other extensions (<Silhouette text=".#.|###|###" />, <Silhouette text="#..|###|###" />) introduce marginal-only solutions, and adding both produces an outright impossible pair. Section 7's compatibility graphs let you build candidate decks by hand and see which ones survive.</p>

<div class="credit">
<strong>Project Cube</strong> is © Alain Rivollet, all rights reserved. This post is an analysis, not a substitute. The fun bit of the puzzle is the head-to-head race against someone else — sitting across from a person staring at the same two cards, both finding a particular puzzle way more difficult than the other. That experience is in the box, not on this page.
</div>
