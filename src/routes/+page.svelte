<script>
  import { base } from '$app/paths';
  import BlogHead from '$lib/BlogHead.svelte';

  // Add a new entry per post.
  const posts = [
    {
      slug: 'project-cube',
      title: 'Project Cube — well-designed or not?',
      date: '2026-05-04',
      summary: 'An analysis of Project Cube by Alain Rivollet: the game, when a 3×3×3 structure balances, which cards extend the deck, and the compatibility graph.',
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

<main class="!mx-auto !mb-24 !mt-0 !w-full !max-w-5xl px-5 py-10 font-sans sm:px-8 sm:py-14">
  <header class="mb-9 max-w-2xl">
    <p class="!mb-2 !mt-0 text-sm font-bold uppercase tracking-[0.18em] !text-blue-600 dark:!text-blue-400">Writing</p>
    <h1 class="!m-0 !text-4xl !font-bold !tracking-tight sm:!text-5xl">Blog</h1>
    <p class="!mt-4 !text-lg !leading-relaxed !text-gray-600 dark:!text-gray-300">
      Notes about mathematical software, algorithms, optimisation, and the projects I build.
    </p>
  </header>

  <ul class="m-0 flex list-none flex-col gap-6 p-0">
    {#each posts as post}
      <li>
        <a
          href={`${base}/${post.slug}/`}
          class="group grid overflow-hidden rounded-2xl border border-gray-200 bg-white no-underline shadow-sm transition hover:-translate-y-0.5 hover:border-blue-300 hover:shadow-md dark:border-gray-700 dark:bg-gray-800 dark:hover:border-blue-600 sm:grid-cols-[minmax(0,1fr)_18rem]"
        >
          <div class="flex flex-col p-6 sm:p-7">
            <time class="mb-2 text-sm font-semibold !text-blue-600 dark:!text-blue-400" datetime={post.date}>{formatDate(post.date)}</time>
            <h2 class="m-0 text-2xl font-bold leading-tight text-gray-900 transition-colors group-hover:text-blue-600 dark:text-gray-100 dark:group-hover:text-blue-400">
              {post.title}
            </h2>
            <p class="!mb-0 !mt-3 !leading-relaxed !text-gray-600 dark:!text-gray-300">{post.summary}</p>
            <span class="mt-auto pt-5 text-sm font-bold text-blue-600 dark:text-blue-400">Read the analysis →</span>
          </div>
          <div class="h-60 w-full overflow-hidden border-t border-gray-200 bg-gray-50 dark:border-gray-700 dark:bg-gray-900 sm:h-full sm:border-l sm:border-t-0">
            <iframe
              src={`${base}/project-cube/tower_view.html?example=easiest_game_pair`}
              title="A stable Project Cube tower matching two silhouette cards"
              class="pointer-events-none h-full w-full border-0"
              tabindex="-1"
            ></iframe>
          </div>
        </a>
      </li>
    {/each}
  </ul>

</main>
