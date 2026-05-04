<script>
  import { base } from '$app/paths';
</script>

<svelte:head><title>The rules — Project Cube</title></svelte:head>

<!-- Section 2 — The rules: how the game is played                   -->
<!-- ============================================================== -->

<h2>The rules</h2>

<p>The box contains <strong>two identical sets</strong> of pieces, one for each player. Each set is six wooden blocks: four solid (a 1×1×3 column, an L-tromino, a 1×1×2, and a single cube), and two hollow (a hollow 1×1×2 and a hollow 1×1×1, which are wooden frames you can see straight through). One set is red, the other blue. There's a deck of fifteen cards, each printed with a 3×3 silhouette, and a wooden holder that stands two cards perpendicular to each other so both silhouettes are visible at once.</p>

<figure>
  <img class="hero" src={`${base}/project-cube/assets/project-cube-box.jpg`} alt="Project Cube box: red and blue blocks, hollow wooden blocks, the dual-card holder, a deck of silhouette cards.">
  <figcaption>What you actually pull out of the box.</figcaption>
</figure>

<figure>
  <iframe class="iframe-pieces" src={`${base}/project-cube/assets/pieces_overview.html`} loading="lazy" title="all six piece types"></iframe>
  <figcaption>The six piece types in one player's set, laid out side by side. Drag to rotate, scroll to zoom. The two pieces in the second row are hollow — you can see right through them.</figcaption>
</figure>

<p>To play: two cards go into the holder, perpendicular to each other so one shows the front silhouette and the other the side. Then both players race. You build a 3-D shape, in your colour, whose front and side projections match the two cards. First stable assembly wins.</p>

<p>Three rules are important:</p>
<ul>
<li><strong>Hollow blocks don't count toward the silhouette.</strong> They sit in the structure but the holes go all the way through, so when you look at a hollow cell from any direction, you see <em>past</em> it. The silhouette only counts the solid (red or blue) blocks. Hollow blocks are pure structural support.</li>
<li><strong>You don't have to use every piece.</strong> The official rules don't actually pin this down. Later, we'll find out the game only works if you don't have to use every piece. Therefore, the makers probably meant for this rule to hold (we'll also look at the case when you have to use every piece).</li>
<li><strong>Cards rotate and reflect.</strong> The card holder is just a wooden slot, any of the four 90° rotations of a card slides in fine. Since the back of the card is the reflection of the front (since your partner who sits somewhere else needs to build the same tower seeing the back) each reflection can also be built per card. Since every piece can be reflected this distinction won't actually matter: if you can build a tower for some pair of cards, you can naturally build its mirror as well.</li>
</ul>

<!-- ============================================================== -->
