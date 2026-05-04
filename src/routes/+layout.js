export const prerender = true;

// Without this, adapter-static emits flat `<route>.html` files. GitHub Pages
// then 404s on `<route>/` URLs (the trailing-slash form) because it expects
// `<route>/index.html`. With trailingSlash = 'always', every page renders to
// `<route>/index.html`, and both `/<route>` and `/<route>/` resolve.
export const trailingSlash = 'always';
