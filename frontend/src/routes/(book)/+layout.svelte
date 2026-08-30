<script lang="ts">
	// Book mode layout: minimal shell with no nav rail.
	// Uses the same theme as the rest of Storyteller.
	let { children } = $props();
</script>

<div class="book-shell">
	{@render children()}
</div>

<style>
	.book-shell {
		display: grid;
		grid-template-columns: 240px 1fr;
		grid-template-rows: auto 1fr;
		height: 100vh;
		min-height: 0;
		background: var(--bg);
	}

	@media (max-width: 768px) {
		.book-shell {
			grid-template-columns: 1fr;
		}
	}
</style>
