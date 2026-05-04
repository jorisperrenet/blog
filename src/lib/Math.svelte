<script>
  import katex from 'katex';
  import 'katex/dist/katex.min.css';

  // `expr` is a LaTeX string. `display` makes it block-level (centred, larger).
  // `katex.renderToString` is synchronous and runs at SSR/prerender time, so the
  // emitted HTML is already in the static page — no client-side render pass.
  let { expr, display = false } = $props();

  let html = $derived(katex.renderToString(String(expr ?? ''), {
    displayMode: display,
    throwOnError: false,
    output: 'html',
  }));
</script>

{#if display}
  <div class="my-2">{@html html}</div>
{:else}
  <span>{@html html}</span>
{/if}

<style>
  /* KaTeX defaults to 1.21em, which is too large against this blog's Georgia
     body text. These rules live here (not post.css) so they bundle into the
     same chunk as katex.min.css and are appended after it — winning the
     same-specificity tiebreaker. */
  :global(.katex) { font-size: 1.02em; }
  :global(.katex-display) { font-size: 1.04em; margin: 0.7em 0; }
</style>

