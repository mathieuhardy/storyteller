<script lang="ts">
	// Book mode layout: minimal shell with no nav rail, dark theme optimized for
	// writing. Only contains the file tree sidebar and main content area.
	import { onMount } from 'svelte';

	let { children } = $props();

	// Apply book theme on mount, restore on destroy.
	onMount(() => {
		document.documentElement.setAttribute('data-book-mode', 'true');
		return () => {
			document.documentElement.removeAttribute('data-book-mode');
		};
	});
</script>

<div class="book-shell">
	{@render children()}
</div>

<style>
	.book-shell {
		display: grid;
		grid-template-columns: 240px 1fr;
		height: 100vh;
		min-height: 0;
		background: var(--book-bg, var(--bg));
	}

	@media (max-width: 768px) {
		.book-shell {
			grid-template-columns: 1fr;
		}
	}
</style>
