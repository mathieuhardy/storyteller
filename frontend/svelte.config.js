import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/**
 * Storyteller ships as a single-user, local-first app: the built front is a
 * static SPA served by `storyteller-server` (see docs/architecture.md §3, M6).
 * `fallback` makes every route resolve to the client-rendered shell.
 * @type {import('@sveltejs/kit').Config}
 */
const config = {
	preprocess: vitePreprocess(),
	kit: {
		adapter: adapter({ fallback: 'index.html' }),
		alias: {
			$components: 'src/lib/components',
			$api: 'src/lib/api',
			$i18n: 'src/lib/i18n'
		}
	}
};

export default config;
