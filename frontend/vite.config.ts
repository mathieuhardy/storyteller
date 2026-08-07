import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

// In dev, the SvelteKit server proxies the API to `storyteller-server` so the
// front talks to a same-origin `/api` and needs no CORS. In production the
// server serves both the built front and the API from one origin.
export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		proxy: {
			'/api': {
				target: 'http://127.0.0.1:8787',
				changeOrigin: true
			}
		}
	}
});
