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
    <h3 class="m-0 mb-2 text-[0.92em] font-normal text-gray-600 dark:text-gray-300">Front silhouette</h3>
    <div class="grid grid-cols-3 gap-1 rounded-md border border-gray-300 bg-white p-2 dark:border-gray-600 dark:bg-gray-800">
      {#each rows as z}
        {#each cols as a}
          {@const bit = bitOf(a, z)}
          <button
            type="button"
            onclick={() => toggleFront(bit)}
            aria-pressed={isOn(frontMask, bit)}
            aria-label={`Front cell column ${a + 1}, row ${z + 1}`}
            class="h-14 w-14 cursor-pointer rounded-sm border transition-colors {isOn(frontMask, bit) ? 'border-gray-900 bg-gray-900 dark:border-gray-100 dark:bg-gray-100' : 'border-gray-300 bg-gray-100 hover:bg-gray-200 dark:border-gray-600 dark:bg-gray-700 dark:hover:bg-gray-600'}"
          ></button>
        {/each}
      {/each}
    </div>
  </div>
  <div>
    <h3 class="m-0 mb-2 text-[0.92em] font-normal text-gray-600 dark:text-gray-300">Side silhouette</h3>
    <div class="grid grid-cols-3 gap-1 rounded-md border border-gray-300 bg-white p-2 dark:border-gray-600 dark:bg-gray-800">
      {#each rows as z}
        {#each cols as a}
          {@const bit = bitOf(a, z)}
          <button
            type="button"
            onclick={() => toggleSide(bit)}
            aria-pressed={isOn(sideMask, bit)}
            aria-label={`Side cell column ${a + 1}, row ${z + 1}`}
            class="h-14 w-14 cursor-pointer rounded-sm border transition-colors {isOn(sideMask, bit) ? 'border-gray-900 bg-gray-900 dark:border-gray-100 dark:bg-gray-100' : 'border-gray-300 bg-gray-100 hover:bg-gray-200 dark:border-gray-600 dark:bg-gray-700 dark:hover:bg-gray-600'}"
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
    class="cursor-pointer rounded-md border border-gray-300 bg-white px-4 py-1.5 text-gray-900 transition-colors hover:border-blue-500 hover:text-blue-600 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100 dark:hover:border-blue-400 dark:hover:text-blue-400"
  >Clear</button>
  <button
    type="button"
    onclick={onRandom}
    disabled={!allKeys?.length}
    class="cursor-pointer rounded-md border border-gray-300 bg-white px-4 py-1.5 text-gray-900 transition-colors hover:border-blue-500 hover:text-blue-600 disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100 dark:hover:border-blue-400 dark:hover:text-blue-400"
  >Random pair</button>
</div>

{#if loadError}
  <div class="my-4 rounded-md border-l-[3px] border-amber-500 bg-amber-50 px-4 py-3 text-[0.95em] text-amber-950 dark:border-amber-400 dark:bg-amber-950/40 dark:text-amber-100">
    Could not load solver database: {loadError}
  </div>
{:else if !dbIndex}
  <div class="my-4 rounded-md bg-gray-100 px-4 py-3 text-[0.95em] italic text-gray-600 dark:bg-gray-800 dark:text-gray-300">
    Loading database…
  </div>
{:else if frontMask === 0 && sideMask === 0}
  <div class="my-4 rounded-md bg-gray-100 px-4 py-3 text-[0.95em] dark:bg-gray-800">
    Loaded {allKeys.length.toLocaleString()} solvable silhouette pairs. Click cells to begin, or hit "Random pair".
  </div>
{:else if entry === 'impossible'}
  <div class="my-4 rounded-md border-l-[3px] border-amber-500 bg-amber-50 px-4 py-3 text-[0.95em] text-amber-950 dark:border-amber-400 dark:bg-amber-950/40 dark:text-amber-100">
    <strong>Impossible.</strong> No arrangement of pieces matches both silhouettes.
  </div>
{:else}
  {@const [, , strict, marginal] = entry}
  <div class="my-4 rounded-md border-l-[3px] border-green-500 bg-green-50 px-4 py-3 text-[0.95em] text-green-950 dark:border-green-400 dark:bg-green-950/40 dark:text-green-100">
    <strong>{strict + marginal}</strong> stable solution{strict + marginal === 1 ? '' : 's'}
    ({strict} strict, {marginal} marginal). The tower below is one of them.
  </div>
{/if}

{#if towerUrl}
  <div class="mt-3 h-[480px] w-full overflow-hidden rounded-lg border border-gray-300 bg-white dark:border-gray-600 dark:bg-gray-800">
    <iframe src={towerUrl} title="tower" class="w-full h-full border-0 rounded-none"></iframe>
  </div>
{:else if entry === 'impossible'}
  <div class="mt-3 w-full rounded-lg border border-gray-300 bg-white px-5 py-8 text-center italic text-gray-600 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300">
    No tower exists for this pair.
  </div>
{:else}
  <div class="mt-3 w-full rounded-lg border border-gray-300 bg-white px-5 py-8 text-center italic text-gray-600 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300">
    Pick a silhouette pair to see a tower…
  </div>
{/if}
