<script>
  import BlogHead from '$lib/BlogHead.svelte';

  // Add a new entry per post.
  const posts = [
    {
      slug: 'project-cube',
      title: 'Project Cube — well-designed or not?',
      date: '2026-05-04',
      summary: 'An analysis of <em>Project Cube</em> by Alain Rivollet: the game, when does a 3×3×3 structure balance, which of the 102 cards form valid extensions to the deck, compatibility graphs.',
      preview: 'project-cube/tower_view.html?example=easiest_game_pair',
    },
  ];

  function formatDate(iso) {
    const [y, m, d] = iso.split('-').map(Number);
    const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
    return `${months[m - 1]} ${d}, ${y}`;
  }
</script>

<BlogHead
  title="Joris Perrenet — blog"
  description="Notes and analyses by Joris Perrenet, including a deep-dive on the Project Cube puzzle game by Alain Rivollet."
  path="/blog/"
/>

<main>
  <h1>Blog</h1>
  <p>Notes and analyses by <a href="https://jorisperrenet.com/">Joris Perrenet</a>.</p>
  <ul class="list-none p-0 mt-8 flex flex-col gap-3">
    {#each posts as post}
      <li>
        <a
          href={`${post.slug}/`}
          class="group block px-5 py-4 rounded-lg border border-border bg-card no-underline transition-all hover:border-accent hover:shadow-sm"
        >
          <span class="block text-[1.15em] font-semibold text-fg group-hover:text-accent transition-colors">{post.title}</span>
          {#if post.preview}
            <iframe
              src={post.preview}
              loading="lazy"
              title={`${post.title} preview`}
              class="float-right w-[180px] h-[180px] ml-4 mt-2 mb-1 rounded-md border border-border bg-bg"
            ></iframe>
          {/if}
          <div class="text-fg mt-1.5 text-[0.95em] leading-relaxed">{@html post.summary}</div>
          <time class="block mt-3 text-[0.85em] text-muted" datetime={post.date}>{formatDate(post.date)}</time>
          <div class="clear-both"></div>
        </a>
      </li>
    {/each}
  </ul>
</main>
