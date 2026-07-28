<script>
  /**
   * Per-page meta for blog posts.
   *  - `title`: shown in browser tab and search results
   *  - `description`: 1-sentence summary for search results / link previews
   *  - `path`: site-relative URL incl. trailing slash, e.g. `/project-cube/rules/`
   *  - `image`: optional absolute preview URL
   *  - `published`: ISO date — emits article:published_time + JSON-LD datePublished
   */
  let {
    title,
    description,
    path,
    image = 'https://jorisperrenet.com/blog/project-cube/assets/tower-preview.png',
    published = null,
  } = $props();

  const origin = 'https://jorisperrenet.com';
  let url = $derived(origin + path);

  let jsonLd = $derived(JSON.stringify({
    '@context': 'https://schema.org',
    '@type': published ? 'BlogPosting' : 'WebPage',
    headline: title,
    description,
    url,
    image,
    ...(published ? { datePublished: published, dateModified: published } : {}),
    author: {
      '@type': 'Person',
      name: 'Joris Perrenet',
      url: 'https://jorisperrenet.com/',
    },
    publisher: {
      '@type': 'Person',
      name: 'Joris Perrenet',
    },
  }));
</script>

<svelte:head>
  <title>{title}</title>
  <meta name="description" content={description} />
  <link rel="canonical" href={url} />
  <meta property="og:type" content={published ? 'article' : 'website'} />
  <meta property="og:title" content={title} />
  <meta property="og:description" content={description} />
  <meta property="og:url" content={url} />
  <meta property="og:image" content={image} />
  <meta property="og:image:width" content="1200" />
  <meta property="og:image:height" content="630" />
  <meta property="og:image:alt" content="A stable Project Cube tower matching two silhouette cards" />
  <meta property="og:site_name" content="Joris Perrenet — blog" />
  {#if published}<meta property="article:published_time" content={published} />{/if}
  <meta name="twitter:card" content="summary_large_image" />
  <meta name="twitter:title" content={title} />
  <meta name="twitter:description" content={description} />
  <meta name="twitter:image" content={image} />
  <meta name="twitter:image:alt" content="A stable Project Cube tower matching two silhouette cards" />
  {@html `<script type="application/ld+json">${jsonLd}<\/script>`}
</svelte:head>
