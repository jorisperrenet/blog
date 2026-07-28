<script>
  import './layout.css';
  import { base } from '$app/paths';
  import SiteShell from '$lib/site-kit/SiteShell.svelte';

  let { children } = $props();
</script>

<SiteShell
  active="blog"
  projectName="Blog"
  currentProjectId="blog"
  projectHref={`${base}/`}
  sourceHref="https://github.com/jorisperrenet/blog"
  brandLogo
  logoSrc={`${base}/personal-logo.svg`}
  localNavigation
>
  {@render children()}
</SiteShell>
