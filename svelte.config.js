import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	compilerOptions: {
		// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
		runes: ({ filename }) => filename.split(/[/\\]/).includes('node_modules') ? undefined : true
	},
	kit: {
		adapter: adapter(),
		prerender: {
			handleHttpError: ({ path, message }) => {
				if (
					path === '/' ||
					path === '/about' ||
					path === '/about/' ||
					path === '/blog' ||
					path === '/blog/'
				) return;
				throw new Error(message);
			}
		},
		paths: {
			base: process.env.BASE_PATH ?? '',
			// `paths.relative: true` (SvelteKit's default) was producing relative
			// hrefs that the client-side router couldn't always resolve when
			// navigating between project-cube sub-pages — content area would
			// stay empty until a hard reload. Absolute paths (with BASE_PATH
			// baked in) sidestep that.
			relative: false
		}
	}
};

export default config;
