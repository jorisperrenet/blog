<script>
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
  let solverActive = $derived(page.url.pathname.includes('/project-cube/solver'));
</script>

<header class="border-b border-border bg-bg/90 backdrop-blur-sm sticky top-0 z-10">
  <nav class="max-w-[720px] mx-auto px-5 py-3 flex flex-wrap items-center gap-x-1.5 gap-y-2 text-[0.85em] tabular-nums">
    <a href={`${base}/project-cube/`} class="text-[1.05em] font-semibold text-fg no-underline whitespace-nowrap mr-1.5">Project Cube</a>
    {#each sections as s}
      {@const active = s.slug === nav?.current?.slug}
      <a
        href={`${base}/project-cube/${s.slug}${s.slug ? '/' : ''}`}
        aria-label={`${s.num}. ${s.title}`}
        class="group inline-flex items-center h-8 rounded-full border no-underline whitespace-nowrap overflow-hidden transition-all duration-200 {active ? 'bg-accent text-white border-accent pr-3 shadow-sm' : 'bg-card text-fg border-border-strong hover:border-accent hover:text-accent hover:pr-3 hover:shadow-sm'}"
      >
        <span class="w-8 text-center font-semibold shrink-0">{s.num}</span>
        <span class="overflow-hidden transition-[max-width] duration-200 {active ? 'max-w-[160px]' : 'max-w-0 group-hover:max-w-[160px]'}">{s.title}</span>
      </a>
    {/each}
    <a
      href={`${base}/project-cube/solver/`}
      class="inline-flex items-center h-8 px-3.5 rounded-full border border-accent font-semibold no-underline whitespace-nowrap transition-colors {solverActive ? 'bg-accent text-white shadow-sm' : 'bg-card text-accent hover:bg-accent hover:text-white hover:shadow-sm'}"
    >Solver</a>
  </nav>
</header>

<main>{@render children()}</main>

{#if nav && (nav.prev || nav.next)}
  <nav aria-label="Section navigation" class="max-w-[640px] mx-auto px-5 mt-12 mb-6 flex justify-between gap-4 text-[0.95em]">
    {#if nav.prev}
      <a
        href={`${base}/project-cube/${nav.prev.slug}${nav.prev.slug ? '/' : ''}`}
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
        href={`${base}/project-cube/${nav.next.slug}${nav.next.slug ? '/' : ''}`}
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
