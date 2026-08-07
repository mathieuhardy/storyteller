<script lang="ts">
	// A 30px square icon button (theme, panel toggle, close…). `active` accents it
	// (docs/ui/components.md §1).
	import type { Snippet } from 'svelte';

	let {
		label,
		active = false,
		disabled = false,
		onclick = undefined,
		children
	}: {
		label: string;
		active?: boolean;
		disabled?: boolean;
		onclick?: (event: MouseEvent) => void;
		children: Snippet;
	} = $props();
</script>

<button class="icon-btn" class:active title={label} aria-label={label} {disabled} {onclick}>
	{@render children()}
</button>

<style>
	.icon-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 30px;
		height: 30px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--muted);
		cursor: pointer;
		transition:
			background 0.12s ease,
			color 0.12s ease,
			border-color 0.12s ease;
	}
	.icon-btn:hover {
		color: var(--text);
		border-color: var(--border-strong);
		background: var(--surface-2);
	}
	.icon-btn.active {
		color: var(--accent);
		border-color: var(--accent-line);
		background: var(--accent-soft);
	}
	.icon-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
