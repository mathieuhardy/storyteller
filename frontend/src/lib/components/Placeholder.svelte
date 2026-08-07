<script lang="ts">
	// Neutral placeholder for screens delivered in later steps of the milestone.
	import Icon from './Icon.svelte';

	let { title, body }: { title: string; body: string } = $props();
</script>

<div class="placeholder">
	<span class="glyph" aria-hidden="true"><Icon name="feather" size={22} /></span>
	<h2>{title}</h2>
	<p>{body}</p>
</div>

<style>
	.placeholder {
		max-width: 460px;
		margin: 12vh auto 0;
		padding: 0 24px;
		text-align: center;
	}
	.glyph {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 52px;
		height: 52px;
		border-radius: var(--radius-lg);
		background: var(--accent-soft);
		color: var(--accent);
		margin-bottom: 14px;
	}
	h2 {
		margin: 0 0 8px;
		font-size: 21px;
		font-weight: 660;
	}
	p {
		margin: 0;
		color: var(--muted);
		font-size: 14px;
		line-height: 1.6;
	}
</style>
