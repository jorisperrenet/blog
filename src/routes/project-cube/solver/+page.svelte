<script>
  import { onMount } from 'svelte';
  import { base } from '$app/paths';
  import BlogHead from '$lib/BlogHead.svelte';

  // The solver_db.json file is the same one the legacy solver.html uses —
  // an array of [front_mask, side_mask, strict, marginal, placements]
  // tuples covering every (front, side) pair that has at least one stable
  // tower (under the piece-skipping-allowed rules).
  let dbIndex = $state(null);
  let allKeys = $state(null);
  let loadError = $state(null);

  let frontMask = $state(0);
  let sideMask = $state(0);

  // Display order: rows from z=2 (top of the silhouette) down to z=0
  // (ground), columns left-to-right. Each cell encodes a single bit of
  // the 9-bit silhouette via bit = a * 3 + z.
  const rows = [2, 1, 0];
  const cols = [0, 1, 2];

  function bitOf(a, z) { return a * 3 + z; }
  function isOn(mask, bit) { return ((mask >> bit) & 1) !== 0; }
  function toggleFront(bit) { frontMask ^= 1 << bit; }
  function toggleSide(bit)  { sideMask  ^= 1 << bit; }

  // Lookup the current pair. `null` = both grids empty (initial state),
  // 'impossible' = pair not in the DB, or the entry from the DB.
  let entry = $derived.by(() => {
    if (frontMask === 0 && sideMask === 0) return null;
    if (!dbIndex) return null;
    const key = (frontMask << 9) | sideMask;
    return dbIndex.get(key) ?? 'impossible';
  });

  // Build the tower_view URL with the tower spec encoded into the query
  // string — same scheme the legacy solver.html used.
  let towerUrl = $derived.by(() => {
    if (!entry || entry === 'impossible') return null;
    const [, , strict, marginal, placements] = entry;
    const stab = strict > 0 ? 'STRICT' : (marginal > 0 ? 'MARGINAL' : 'UNSTABLE');
    const towerSpec = {
      front: frontMask, side: sideMask, strict, marginal, stab,
      tiles: placements.map(([type, cells]) => ({ type, cells })),
    };
    const b64 = btoa(JSON.stringify(towerSpec))
      .replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
    return `${base}/project-cube/tower_view.html?data=${b64}`;
  });

  function onClear() { frontMask = 0; sideMask = 0; }
  function onRandom() {
    if (!allKeys?.length) return;
    const k = allKeys[Math.floor(Math.random() * allKeys.length)];
    frontMask = (k >> 9) & 0x1ff;
    sideMask  = k & 0x1ff;
  }

  onMount(async () => {
    try {
      const r = await fetch(`${base}/project-cube/assets/solver_db.json`);
      if (!r.ok) throw new Error(`HTTP ${r.status}`);
      const arr = await r.json();
      const idx = new Map();
      const keys = [];
      for (const e of arr) {
        const k = (e[0] << 9) | e[1];
        idx.set(k, e);
        keys.push(k);
      }
      dbIndex = idx;
      allKeys = keys;
      onRandom();
    } catch (err) {
      loadError = String(err);
    }
  });
</script>

<BlogHead
  title="Silhouette pair solver — Project Cube"
  description="Live solver for Project Cube: pick any front and side silhouette pair and see a stable 3-D wooden tower that satisfies both views."
  path="/blog/project-cube/solver/"
/>

<h1>Silhouette pair solver</h1>

<p>Toggle cells in the front and side silhouettes below; the solver tells you whether that pair is solvable, how many stable arrangements exist, and shows one of them as a 3-D tower. All 72,994 solvable pairs are pre-computed; pairs not in that set don't admit any stable solution under the actual game's rules (piece-skipping allowed).</p>

<div class="flex gap-6 flex-wrap items-start my-6">
  <div>
    <h3 class="text-[0.92em] text-muted m-0 mb-2 font-normal">Front silhouette</h3>
    <div class="grid grid-cols-3 gap-1 p-2 bg-card rounded-md border border-border-strong">
      {#each rows as z}
        {#each cols as a}
          {@const bit = bitOf(a, z)}
          <button
            type="button"
            onclick={() => toggleFront(bit)}
            aria-pressed={isOn(frontMask, bit)}
            aria-label={`Front cell column ${a + 1}, row ${z + 1}`}
            class="w-14 h-14 rounded-sm cursor-pointer transition-colors {isOn(frontMask, bit) ? 'bg-fg border border-fg' : 'bg-code-bg border border-border-strong hover:bg-border'}"
          ></button>
        {/each}
      {/each}
    </div>
  </div>
  <div>
    <h3 class="text-[0.92em] text-muted m-0 mb-2 font-normal">Side silhouette</h3>
    <div class="grid grid-cols-3 gap-1 p-2 bg-card rounded-md border border-border-strong">
      {#each rows as z}
        {#each cols as a}
          {@const bit = bitOf(a, z)}
          <button
            type="button"
            onclick={() => toggleSide(bit)}
            aria-pressed={isOn(sideMask, bit)}
            aria-label={`Side cell column ${a + 1}, row ${z + 1}`}
            class="w-14 h-14 rounded-sm cursor-pointer transition-colors {isOn(sideMask, bit) ? 'bg-fg border border-fg' : 'bg-code-bg border border-border-strong hover:bg-border'}"
          ></button>
        {/each}
      {/each}
    </div>
  </div>
</div>

<div class="flex gap-2 my-3">
  <button
    type="button"
    onclick={onClear}
    class="px-4 py-1.5 rounded-md border border-border-strong bg-card text-fg cursor-pointer hover:border-accent hover:text-accent transition-colors"
  >Clear</button>
  <button
    type="button"
    onclick={onRandom}
    disabled={!allKeys?.length}
    class="px-4 py-1.5 rounded-md border border-border-strong bg-card text-fg cursor-pointer hover:border-accent hover:text-accent transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
  >Random pair</button>
</div>

{#if loadError}
  <div class="my-4 px-4 py-3 rounded-md text-[0.95em] bg-[#fff3cd] border-l-[3px] border-[#ddb44c]">
    Could not load solver database: {loadError}
  </div>
{:else if !dbIndex}
  <div class="my-4 px-4 py-3 rounded-md bg-code-bg text-[0.95em] italic text-muted">
    Loading database…
  </div>
{:else if frontMask === 0 && sideMask === 0}
  <div class="my-4 px-4 py-3 rounded-md bg-code-bg text-[0.95em]">
    Loaded {allKeys.length.toLocaleString()} solvable silhouette pairs. Click cells to begin, or hit "Random pair".
  </div>
{:else if entry === 'impossible'}
  <div class="my-4 px-4 py-3 rounded-md text-[0.95em] bg-[#fff3cd] border-l-[3px] border-[#ddb44c]">
    <strong>Impossible.</strong> No arrangement of pieces matches both silhouettes.
  </div>
{:else}
  {@const [, , strict, marginal] = entry}
  <div class="my-4 px-4 py-3 rounded-md text-[0.95em] bg-[#e6f7e6] border-l-[3px] border-[#5cb85c]">
    <strong>{strict + marginal}</strong> stable solution{strict + marginal === 1 ? '' : 's'}
    ({strict} strict, {marginal} marginal). The tower below is one of them.
  </div>
{/if}

{#if towerUrl}
  <div class="w-full h-[480px] mt-3 rounded-lg border border-border-strong overflow-hidden bg-card">
    <iframe src={towerUrl} title="tower" class="w-full h-full border-0 rounded-none"></iframe>
  </div>
{:else if entry === 'impossible'}
  <div class="w-full px-5 py-8 mt-3 rounded-lg border border-border-strong bg-card italic text-muted text-center">
    No tower exists for this pair.
  </div>
{:else}
  <div class="w-full px-5 py-8 mt-3 rounded-lg border border-border-strong bg-card italic text-muted text-center">
    Pick a silhouette pair to see a tower…
  </div>
{/if}
