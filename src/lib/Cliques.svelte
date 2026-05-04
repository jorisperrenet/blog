<script>
  import Silhouette from './Silhouette.svelte';

  let { data } = $props();
</script>

{#each data as clique}
  <div class="clique-block">
    <h4>
      <span>{clique.title}</span>
      <span class="min-stab">min STRICT = {clique.minStable}</span>
    </h4>
    <div class="clique-cards">
      {#each clique.items as [pat, gid]}
        <div class="clique-card">
          <div style="line-height:0">
            <Silhouette text={pat} size={38} />
          </div>
          <div class="gid {gid ? '' : 'empty'}">{gid ? `card ${gid}` : '—'}</div>
        </div>
      {/each}
    </div>
  </div>
{/each}
