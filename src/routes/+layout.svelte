<script>
  import './layout.css';

  let { children } = $props();
</script>

{@render children()}

<footer class="max-w-[640px] mx-auto px-5 pb-6 text-center text-[0.88em] text-muted">
  © Joris Perrenet · <a href="https://github.com/jorisperrenet/blog" target="_blank" rel="noopener noreferrer">github.com/jorisperrenet/blog</a>
</footer>
