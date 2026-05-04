<script>
  import { afterNavigate } from '$app/navigation';
  import { page } from '$app/state';
  import { base } from '$app/paths';
  import { sections, findSection } from '$lib/sections.js';
  import './post.css';

  let { children } = $props();
  // The route ID under SvelteKit is `/project-cube/<slug>` regardless of the
  // base path. Find the post prefix in the pathname and pass the slug to
  // findSection (which strips slashes itself).
  function postSlug(pathname) {
    const i = pathname.indexOf('/project-cube');
    return i < 0 ? '' : pathname.slice(i + '/project-cube'.length);
  }
  let nav = $derived(findSection(postSlug(page.url.pathname)));
  let solverActive = $derived(
    page.url.pathname.endsWith('/project-cube/solver') ||
    page.url.pathname.includes('/project-cube/solver/')
  );

  // Maximal-clique data, formerly defined as window.CLIQUES_DATA / CLIQUES_ALL_DATA
  // by inline <script> blocks in the original index.html. Now ordinary JS
  // constants used by renderCliques() to populate Section 7.
  const CLIQUES_DATA = [
    {
      title: "Clique 1 (15 cards)",
      minStable: 1,
      items: [
        ["#..|#..|###", "8"], ["#..|##.|.##", "14"], ["#..|##.|###", "4"],
        ["##.|#..|###", null], ["##.|#.#|.##", null], ["##.|#.#|###", "13"],
        ["##.|###|#.#", null], ["##.|###|###", null], ["##.|###|.##", "6"],
        [".#.|.#.|###", "10"], [".#.|##.|###", "1,3"], [".#.|###|##.", "2"],
        ["#.#|##.|###", "15"], ["###|##.|###", null], ["###|#.#|###", null],
      ],
    },
    {
      title: "Clique 2 (15 cards)",
      minStable: 1,
      items: [
        ["#..|#..|###", "8"], ["#..|##.|.##", "14"], ["#..|##.|###", "4"],
        [".#.|.#.|###", "10"], [".#.|##.|.##", "9,11"], [".#.|##.|###", "1,3"],
        [".#.|###|.#.", "12"], [".#.|###|##.", "2"], ["##.|#.#|###", "13"],
        ["##.|###|#.#", null], ["##.|###|###", null], ["##.|###|.##", "6"],
        ["#.#|##.|###", "15"], ["###|##.|###", null], ["###|#.#|###", null],
      ],
    },
    {
      title: "Clique 3 (15 cards)",
      minStable: 1,
      items: [
        ["#..|#..|###", "8"], ["#..|##.|.##", "14"], ["#..|##.|###", "4"],
        [".#.|.#.|###", "10"], [".#.|##.|###", "1,3"], [".#.|###|.#.", "12"],
        [".#.|###|##.", "2"], ["##.|#.#|.##", null], ["##.|#.#|###", "13"],
        ["##.|###|#.#", null], ["##.|###|###", null], ["##.|###|.##", "6"],
        ["#.#|##.|###", "15"], ["###|##.|###", null], ["###|#.#|###", null],
      ],
    },
  ];

  const CLIQUES_ALL_DATA = [
    {
      title: "Clique 1, all-pieces (13 cards)",
      minStable: 1,
      items: [
        ["#..|##.|###", "4"], ["##.|#..|###", null], ["##.|#.#|.##", null],
        ["##.|#.#|###", "13"], ["##.|###|#.#", null], ["##.|###|.##", "6"],
        ["##.|###|###", null], [".#.|##.|###", "1,3"], [".#.|###|##.", "2"],
        ["#.#|#..|###", null], ["#.#|##.|###", "15"], ["###|##.|###", null],
        ["###|#.#|###", null],
      ],
    },
    {
      title: "Clique 2, all-pieces (12 cards)",
      minStable: 1,
      items: [
        ["#..|##.|###", "4"], ["##.|#..|###", null], ["##.|.#.|###", null],
        ["##.|#.#|.##", null], ["##.|#.#|###", "13"], ["##.|###|#.#", null],
        ["##.|###|.##", "6"], [".#.|##.|###", "1,3"], [".#.|###|##.", "2"],
        ["#.#|#..|###", null], ["#.#|##.|###", "15"], ["###|#.#|###", null],
      ],
    },
    {
      title: "Clique 3, all-pieces (12 cards)",
      minStable: 1,
      items: [
        ["#..|##.|###", "4"], ["##.|#..|###", null], ["##.|.##|#.#", null],
        ["##.|#.#|###", "13"], ["##.|###|.##", "6"], ["##.|###|###", null],
        [".#.|##.|###", "1,3"], [".#.|###|##.", "2"], ["#.#|#..|###", null],
        ["#.#|##.|###", "15"], ["###|##.|###", null], ["###|#.#|###", null],
      ],
    },
  ];

  function silhouetteSvg(text, sizePx) {
    const cleaned = text.replace(/\s/g, '');
    if (!/^[#.]+(\|[#.]+)*$/.test(cleaned)) return null;
    const rows = cleaned.split('|');
    if (rows.length !== 3 || rows.some(r => r.length !== 3)) return null;
    const cell = (sizePx - 4) / 3;
    let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${sizePx}" height="${sizePx}" viewBox="0 0 ${sizePx} ${sizePx}" style="border:1px solid #888;border-radius:2px;background:#fffbf0">`;
    for (let z = 0; z < 3; z++) {
      const row = rows[2 - z];
      for (let a = 0; a < 3; a++) {
        if (row[a] === '#') {
          const x = 2 + a * cell;
          const y = 2 + (2 - z) * cell;
          svg += `<rect x="${x}" y="${y}" width="${cell}" height="${cell}" fill="#3a3a3a"/>`;
        }
      }
    }
    svg += '</svg>';
    return svg;
  }

  function maskToText(mask) {
    let s = '';
    for (let z = 2; z >= 0; z--) {
      for (let a = 0; a < 3; a++) {
        s += ((mask >> (a * 3 + z)) & 1) ? '#' : '.';
      }
      if (z > 0) s += '|';
    }
    return s;
  }

  function inlineSilhouettes() {
    document.querySelectorAll('.silh').forEach(el => {
      if (el.dataset.silhRendered) return;
      const svg = silhouetteSvg(el.textContent.trim(), 22);
      if (svg) {
        el.innerHTML = svg;
        el.dataset.silhRendered = '1';
      }
    });
  }

  function loadStats() {
    const stats = window.STATS;
    if (!stats) return;
    document.querySelectorAll('[data-stat]').forEach(el => {
      const parts = el.dataset.stat.split('.');
      let v = stats;
      for (const p of parts) {
        if (v && Object.prototype.hasOwnProperty.call(v, p)) v = v[p];
        else { v = null; break; }
      }
      if (v !== null) el.textContent = String(v);
    });
    function setSilhouette(elId, mask) {
      const host = document.getElementById(elId);
      if (host && typeof mask === 'number') {
        host.innerHTML = silhouetteSvg(maskToText(mask), 22);
        host.dataset.silhRendered = '1';
      }
    }
    if (stats.easiest_game) {
      setSilhouette('easiest-game-front', stats.easiest_game.front_mask);
      setSilhouette('easiest-game-side',  stats.easiest_game.side_mask);
    }
    if (stats.skip_required) {
      setSilhouette('skip-required-front', stats.skip_required.front_mask);
      setSilhouette('skip-required-side',  stats.skip_required.side_mask);
    }
  }

  function loadSolver() {
    const pairs = window.HARDPAIRS_MANIFEST;
    if (!pairs) return;
    const select = document.getElementById('solver-select');
    if (!select) return; // Solver only lives on the /pairs page.
    if (select.options.length > 0) return; // Idempotent: already wired up.
    const loading = document.getElementById('solver-loading');
    const ui = document.getElementById('solver-ui');
    const frontHost = document.getElementById('solver-front-svg');
    const sideHost = document.getElementById('solver-side-svg');
    const iframe = document.getElementById('solver-iframe');
    const answer = document.getElementById('solver-answer');
    const toggle = document.getElementById('solver-toggle');
    if (!pairs.length) { loading.textContent = '(no pairs found)'; return; }

    pairs.forEach((p, i) => {
      const opt = document.createElement('option');
      opt.value = String(i);
      const marg = p.marginal > 0 ? ` (+${p.marginal} marginal)` : '';
      opt.textContent = `card ${p.front.card} (${p.front.rot}) × card ${p.side.card} (${p.side.rot})${marg}`;
      select.appendChild(opt);
    });

    function renderPair(idx) {
      const p = pairs[idx];
      frontHost.innerHTML = silhouetteSvg(maskToText(p.front.mask), 64);
      sideHost.innerHTML = silhouetteSvg(maskToText(p.side.mask), 64);
      iframe.src = '';
      answer.classList.add('hidden');
      toggle.textContent = 'Show solution';
    }

    select.addEventListener('change', () => renderPair(parseInt(select.value)));
    toggle.addEventListener('click', () => {
      const p = pairs[parseInt(select.value)];
      if (answer.classList.contains('hidden')) {
        const exampleName = p.file.replace(/\.html$/, '');
        iframe.src = 'tower_view.html?example=' + exampleName;
        answer.classList.remove('hidden');
        toggle.textContent = 'Hide solution';
      } else {
        answer.classList.add('hidden');
        iframe.src = '';
        toggle.textContent = 'Show solution';
      }
    });

    renderPair(0);
    loading.classList.add('hidden');
    ui.classList.remove('hidden');
  }

  function renderCliquesInto(hostId, data) {
    const host = document.getElementById(hostId);
    if (!host || !data) return;
    if (host.children.length > 0) return; // Idempotent: already populated.
    for (const clique of data) {
      const block = document.createElement('div');
      block.className = 'clique-block';
      const h = document.createElement('h4');
      const titleSpan = document.createElement('span');
      titleSpan.textContent = clique.title;
      h.appendChild(titleSpan);
      const stab = document.createElement('span');
      stab.className = 'min-stab';
      stab.textContent = `min STRICT = ${clique.minStable}`;
      h.appendChild(stab);
      block.appendChild(h);
      const cards = document.createElement('div');
      cards.className = 'clique-cards';
      for (const [pat, gid] of clique.items) {
        const card = document.createElement('div');
        card.className = 'clique-card';
        const svgHost = document.createElement('div');
        svgHost.style.lineHeight = 0;
        svgHost.innerHTML = silhouetteSvg(pat, 38);
        card.appendChild(svgHost);
        const idEl = document.createElement('div');
        idEl.className = gid ? 'gid' : 'gid empty';
        idEl.textContent = gid ? `card ${gid}` : '—';
        card.appendChild(idEl);
        cards.appendChild(card);
      }
      block.appendChild(cards);
      host.appendChild(block);
    }
  }

  function renderCliques() {
    renderCliquesInto('cliques-host', CLIQUES_DATA);
    renderCliquesInto('cliques-all-host', CLIQUES_ALL_DATA);
  }

  // KaTeX, stats.js and hardpairs_manifest.js are loaded from app.html with
  // `defer`. Poll briefly to cover slow networks / dev races, then run the
  // same setup the IIFE in the original index.html ran on DOMContentLoaded.
  function whenReady(check, cb, attempts = 60) {
    if (check()) return cb();
    if (attempts <= 0) return;
    setTimeout(() => whenReady(check, cb, attempts - 1), 50);
  }

  function setupAll() {
    inlineSilhouettes();
    whenReady(() => window.STATS, loadStats);
    whenReady(() => window.HARDPAIRS_MANIFEST, loadSolver);
    renderCliques();
    whenReady(() => window.renderMathInElement, () => {
      window.renderMathInElement(document.body, {
        delimiters: [
          { left: '$$', right: '$$', display: true },
          { left: '\\(', right: '\\)', display: false },
        ],
      });
    });
  }

  // Runs after every navigation, including the initial page load — keeps
  // the inline silhouettes / solver / cliques / KaTeX in sync as you move
  // between section pages.
  afterNavigate(setupAll);
</script>

<header class="border-b border-border bg-bg/90 backdrop-blur-sm sticky top-0 z-10">
  <nav class="max-w-[720px] mx-auto px-5 py-3 flex flex-wrap items-center gap-x-1.5 gap-y-2 text-[0.85em] tabular-nums">
    <a href={`${base}/project-cube/`} class="text-[1.05em] font-semibold text-fg no-underline whitespace-nowrap mr-1.5">Project Cube</a>
    {#each sections as s}
      {@const active = s.slug === nav?.current?.slug}
      <a
        href={`${base}/project-cube/${s.slug}`}
        aria-label={`${s.num}. ${s.title}`}
        class="group inline-flex items-center h-8 rounded-full border no-underline whitespace-nowrap overflow-hidden transition-all duration-200 {active ? 'bg-accent text-white border-accent pr-3 shadow-sm' : 'bg-card text-fg border-border-strong hover:border-accent hover:text-accent hover:pr-3 hover:shadow-sm'}"
      >
        <span class="w-8 text-center font-semibold shrink-0">{s.num}</span>
        <span class="overflow-hidden transition-[max-width] duration-200 {active ? 'max-w-[160px]' : 'max-w-0 group-hover:max-w-[160px]'}">{s.title}</span>
      </a>
    {/each}
    <a
      href={`${base}/project-cube/solver`}
      class="inline-flex items-center h-8 px-3.5 rounded-full border border-accent font-semibold no-underline whitespace-nowrap transition-colors {solverActive ? 'bg-accent text-white shadow-sm' : 'bg-card text-accent hover:bg-accent hover:text-white hover:shadow-sm'}"
    >Solver</a>
  </nav>
</header>

<main>{@render children()}</main>

{#if nav && (nav.prev || nav.next)}
  <nav aria-label="Section navigation" class="max-w-[640px] mx-auto px-5 mt-12 mb-6 flex justify-between gap-4 text-[0.95em]">
    {#if nav.prev}
      <a
        href={`${base}/project-cube/${nav.prev.slug}`}
        class="flex-1 min-w-0 px-4 py-3 rounded-lg border border-border bg-card no-underline hover:border-accent hover:bg-code-bg transition-colors"
      >
        <span class="block text-[0.8em] uppercase tracking-wide text-muted">← Previous</span>
        <span class="block text-fg font-semibold mt-0.5 truncate">{nav.prev.num}. {nav.prev.title}</span>
      </a>
    {:else}
      <span class="flex-1"></span>
    {/if}
    {#if nav.next}
      <a
        href={`${base}/project-cube/${nav.next.slug}`}
        class="flex-1 min-w-0 px-4 py-3 rounded-lg border border-border bg-card no-underline hover:border-accent hover:bg-code-bg transition-colors text-right"
      >
        <span class="block text-[0.8em] uppercase tracking-wide text-muted">Next →</span>
        <span class="block text-fg font-semibold mt-0.5 truncate">{nav.next.num}. {nav.next.title}</span>
      </a>
    {:else}
      <span class="flex-1"></span>
    {/if}
  </nav>
{/if}

