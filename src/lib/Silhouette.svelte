<script>
  import { silhouetteSvg, maskToText } from './silhouette.js';

  // Provide either `text` (e.g. "##.|###|##.") or `mask` (9-bit integer).
  // `size` defaults to 22 (inline body) — pass 38/64 for larger callouts.
  let { text, mask, size = 22 } = $props();

  let resolved = $derived(text ?? (typeof mask === 'number' ? maskToText(mask) : ''));
  let svg = $derived(silhouetteSvg(resolved, size));
</script>

{#if svg}
  <span class="inline-block align-middle leading-none" style="font-size:0">{@html svg}</span>
{:else}
  <code>{resolved || '?'}</code>
{/if}
