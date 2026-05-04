<script>
  import { base } from '$app/paths';
  import Silhouette from '$lib/Silhouette.svelte';

  let { data } = $props();
  const { stats, hardpairs } = data;

  // Solver widget state: which pair is selected and whether the solution is shown.
  let pickIndex = $state(0);
  let showAnswer = $state(false);

  let pick = $derived(hardpairs[pickIndex]);
  let solutionUrl = $derived(pick
    ? `${base}/project-cube/tower_view.html?example=${pick.file.replace(/\.html$/, '')}`
    : null);

  function toggleAnswer() {
    showAnswer = !showAnswer;
  }

  function onSelect(e) {
    pickIndex = parseInt(e.currentTarget.value, 10);
    showAnswer = false;
  }
</script>

<svelte:head><title>The deck — Project Cube</title></svelte:head>

<h2>The deck</h2>

<p>Great. The enumeration now works. The stability can now also be checked given the configuration. Thus, for each configuration we check whether it stands. If it stands, check the front and side view for overlap with the existing cards of the game (we first store any front + side view and view the ones in the game as a subset of those). Also, note that there are three labels for stability:</p>
<ul>
<li><strong>Strict</strong> — the tower stands without the need of a "perfect world" in which you can balance a 1×1×2 block on a 1×1×1 block.</li>
<li><strong>Marginal</strong> — the tower stands, but only in a "perfect world", without any margin.</li>
<li><strong>Impossible</strong> — the configuration does not stand.</li>
</ul>

<p>Running this enumeration over all piece configurations yields 72,994 distinct (front, side) pairs that admit at least one strict tower, with about 1.6 million strict arrangements summed across all of them. If you want to play with the result, jump to the <a href={`${base}/project-cube/solver/`}><strong>silhouette pair solver</strong></a> — toggle any front/side pair and see whether it's solvable, how many arrangements work, and a 3-D rendering of one of them.</p>

<h3>The easiest pair</h3>

<aside class="note">
  <strong>Heads-up on terminology.</strong> "Easiest" and "hardest" here mean the <em>most</em> and <em>fewest</em> strict arrangements respectively — not how easy or hard a pair is to solve cognitively.
</aside>

<p>This is the pair with the most strict solutions of all 72,994 pairs.</p>

<div class="stat-block">
front <Silhouette text="...|###|###" /> &nbsp;|&nbsp; side <Silhouette text="...|###|###" /> &nbsp;→&nbsp; <strong>2,912</strong> strict solutions
</div>

<p>Those silhouettes aren't actually in the printed deck though. If we restrict to pairs you can actually construct in the game, the easiest one is this:</p>

{#if stats.easiest_game}
  <div class="stat-block">
    front <Silhouette mask={stats.easiest_game.front_mask} /> (card {stats.easiest_game.front_card}, {stats.easiest_game.front_rot})
    &nbsp;|&nbsp;
    side <Silhouette mask={stats.easiest_game.side_mask} /> (card {stats.easiest_game.side_card}, {stats.easiest_game.side_rot})
    &nbsp;→&nbsp; <strong>{stats.easiest_game.strict}</strong> strict solutions
  </div>
{/if}

<figure>
  <iframe class="iframe-tower" src={`${base}/project-cube/tower_view.html?example=easiest_game_pair`} loading="lazy" title="easiest game pair"></iframe>
  <figcaption>One of many strict assemblies for the easiest game pair.</figcaption>
</figure>

<h3>A pair only solvable by skipping a piece</h3>

<p>One of the rules from earlier: you don't have to use every piece. That isn't just a convenience. There are silhouette pairs whose only strict solutions <em>require</em> at least one piece to be left in the box.</p>

{#if stats.skip_required}
  <div class="stat-block">
    front <Silhouette mask={stats.skip_required.front_mask} /> (card {stats.skip_required.front_card}, {stats.skip_required.front_rot})
    &nbsp;|&nbsp;
    side <Silhouette mask={stats.skip_required.side_mask} /> (card {stats.skip_required.side_card}, {stats.skip_required.side_rot})
  </div>
{/if}

<figure>
  <iframe class="iframe-tower" src={`${base}/project-cube/tower_view.html?example=skip_required`} loading="lazy" title="skip-required pair"></iframe>
  <figcaption>A pair from the printed deck — both silhouettes are rotations of actual game cards — but every strict solution leaves at least one piece in the box.</figcaption>
</figure>

<p>Thus, you see that the rule should be made clear: <strong>not all pieces need to be used</strong>.</p>

<h3>Try the hardest pairs yourself</h3>

<p>At the other extreme, there are pairs with exactly one strict solution. Pick a pair below; the front and side silhouettes appear, and you can attempt to build the figure before viewing the solution.</p>

<div class="solver">
  <h3>Hardest game-card pairs (1 strict solution each)</h3>
  <p>
    <label for="solver-select">Pick a pair: </label>
    <select id="solver-select" value={String(pickIndex)} onchange={onSelect}>
      {#each hardpairs as p, i}
        <option value={String(i)}>
          card {p.front.card} ({p.front.rot}) × card {p.side.card} ({p.side.rot}){p.marginal > 0 ? ` (+${p.marginal} marginal)` : ''}
        </option>
      {/each}
    </select>
  </p>
  {#if pick}
    <div class="pair-display">
      <div>
        <span class="pair-label">front</span><br>
        <span class="silh-large"><Silhouette mask={pick.front.mask} size={64} /></span>
      </div>
      <div class="text-[1.5em] text-subtle">×</div>
      <div>
        <span class="pair-label">side</span><br>
        <span class="silh-large"><Silhouette mask={pick.side.mask} size={64} /></span>
      </div>
    </div>
    <button onclick={toggleAnswer}>{showAnswer ? 'Hide solution' : 'Show solution'}</button>
    {#if showAnswer && solutionUrl}
      <div class="answer">
        <iframe src={solutionUrl} loading="lazy" title="solution"></iframe>
      </div>
    {/if}
  {/if}
</div>

<h3 class="text-[1.4em] mt-10">Extending the original deck</h3>

<p>Two silhouettes that can be added to the original game without breaking anything:</p>

<div class="stat-block">
<Silhouette text="##.|###|###" /> &nbsp;and&nbsp; <Silhouette text="###|#.#|###" />
</div>

<p>Two other silhouettes can also be added. Adding either one introduces specific (front, side) pairs whose only solutions are marginal — they balance on the literal edge. If both are added, then there will be an impossible pair in the deck: front <Silhouette text=".#.|###|###" /> with side <Silhouette text="###|###|#.." />.</p>

<div class="stat-block">
<Silhouette text=".#.|###|###" /> &nbsp;and&nbsp; <Silhouette text="#..|###|###" />
</div>

<figure>
  <iframe class="iframe-tower" src={`${base}/project-cube/tower_view.html?example=extend_marg_dotcenter`} loading="lazy" title="marginal — adding .#.|###|###"></iframe>
  <figcaption>Adding <Silhouette text=".#.|###|###" /> to the deck creates a pair whose only matching tower is marginal.</figcaption>
</figure>

<figure>
  <iframe class="iframe-tower" src={`${base}/project-cube/tower_view.html?example=extend_marg_corner`} loading="lazy" title="marginal — adding #..|###|###"></iframe>
  <figcaption>Adding <Silhouette text="#..|###|###" /> has the same problem on a different pair.</figcaption>
</figure>
