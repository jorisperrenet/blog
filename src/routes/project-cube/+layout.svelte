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

<nav aria-label="Project Cube chapters" class="sticky top-[40px] z-40 border-b border-gray-200 bg-[#f9fbff]/95 px-3 py-2 shadow-sm backdrop-blur-md dark:border-gray-800 dark:bg-[#111827]/95">
  <div class="mx-auto flex max-w-4xl items-center text-xs tabular-nums sm:px-2">
    <details class="group relative mr-1 shrink-0">
      <summary class="flex cursor-pointer list-none items-center gap-1 whitespace-nowrap rounded-lg px-2 py-1.5 font-bold text-gray-900 transition-colors hover:bg-gray-100 hover:text-blue-600 dark:text-gray-100 dark:hover:bg-gray-800 dark:hover:text-blue-400 [&::-webkit-details-marker]:hidden">
        <span>Project Cube</span>
        <svg viewBox="0 0 20 20" class="h-3.5 w-3.5 transition-transform group-open:rotate-180" aria-hidden="true">
          <path d="m5 7.5 5 5 5-5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </summary>
      <div class="absolute left-0 top-full z-50 mt-2 w-72 rounded-xl border border-gray-200 bg-white p-1.5 text-left shadow-xl dark:border-gray-700 dark:bg-gray-800">
        <a href={`${base}/`} class="block rounded-lg px-3 py-2 text-sm font-medium text-gray-700 no-underline transition-colors hover:bg-gray-100 hover:text-black dark:text-gray-200 dark:hover:bg-gray-700 dark:hover:text-white">All blog posts</a>
        <a href={`${base}/project-cube/`} aria-current="page" class="block rounded-lg bg-blue-50 px-3 py-2 text-sm font-medium text-blue-700 no-underline dark:bg-blue-950/50 dark:text-blue-300">Project Cube — well-designed or not?</a>
      </div>
    </details>
    <div class="flex min-w-0 items-center gap-1.5 overflow-x-auto">
    {#each sections as s}
      {@const active = s.slug === nav?.current?.slug}
      <a
        href={`${base}/project-cube/${s.slug}${s.slug ? '/' : ''}`}
        aria-label={`${s.num}. ${s.title}`}
        aria-current={active ? 'page' : undefined}
        title={`${s.num}. ${s.title}`}
        class="inline-flex h-8 min-w-8 shrink-0 items-center justify-center rounded-full border px-2.5 font-semibold no-underline transition-colors {active ? 'border-blue-600 bg-blue-600 text-white' : 'border-gray-300 bg-white text-gray-700 hover:border-blue-500 hover:text-blue-600 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-200 dark:hover:border-blue-400 dark:hover:text-blue-400'}"
      >
        {s.num}
      </a>
    {/each}
    <a
      href={`${base}/project-cube/solver/`}
      aria-current={solverActive ? 'page' : undefined}
      class="inline-flex h-8 shrink-0 items-center whitespace-nowrap rounded-full border px-3 font-semibold no-underline transition-colors {solverActive ? 'border-blue-600 bg-blue-600 text-white' : 'border-blue-500 bg-white text-blue-600 hover:bg-blue-600 hover:text-white dark:bg-gray-800 dark:text-blue-400 dark:hover:bg-blue-600 dark:hover:text-white'}"
    >Solver</a>
    </div>
  </div>
</nav>

<article class="project-cube-article prose prose-gray mx-auto mb-12 mt-10 max-w-[680px] px-5 prose-a:text-blue-600 prose-a:decoration-blue-300 hover:prose-a:text-blue-700 dark:prose-invert dark:prose-a:text-blue-400 dark:prose-a:decoration-blue-700 dark:hover:prose-a:text-blue-300">
  {@render children()}
</article>

{#if nav && (nav.prev || nav.next)}
  <nav aria-label="Section navigation" class="max-w-[640px] mx-auto px-5 mt-12 mb-6 flex justify-between gap-4 text-[0.95em]">
    {#if nav.prev}
      <a
        href={`${base}/project-cube/${nav.prev.slug}${nav.prev.slug ? '/' : ''}`}
        class="min-w-0 flex-1 rounded-lg border border-gray-200 bg-white px-4 py-3 no-underline transition-colors hover:border-blue-500 hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:hover:border-blue-400 dark:hover:bg-gray-900"
      >
        <span class="block text-[0.8em] uppercase tracking-wide text-gray-500 dark:text-gray-400">← Previous</span>
        <span class="mt-0.5 block truncate font-semibold text-gray-900 dark:text-gray-100">{nav.prev.num}. {nav.prev.title}</span>
      </a>
    {:else}
      <span class="flex-1"></span>
    {/if}
    {#if nav.next}
      <a
        href={`${base}/project-cube/${nav.next.slug}${nav.next.slug ? '/' : ''}`}
        class="min-w-0 flex-1 rounded-lg border border-gray-200 bg-white px-4 py-3 text-right no-underline transition-colors hover:border-blue-500 hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:hover:border-blue-400 dark:hover:bg-gray-900"
      >
        <span class="block text-[0.8em] uppercase tracking-wide text-gray-500 dark:text-gray-400">Next →</span>
        <span class="mt-0.5 block truncate font-semibold text-gray-900 dark:text-gray-100">{nav.next.num}. {nav.next.title}</span>
      </a>
    {:else}
      <span class="flex-1"></span>
    {/if}
  </nav>
{/if}
