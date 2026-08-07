<script lang="ts">
	// The `.btn` primitive (docs/ui/components.md §1): primary (accent), ghost
	// (surface + border), stub (stub tint). Renders an <a> when `href` is set,
	// otherwise a <button>. Utilities are contained here, not in call sites
	// (ADR 0013).
	import type { Snippet } from 'svelte';

	let {
		variant = 'ghost',
		size = 'md',
		href = undefined,
		type = 'button',
		disabled = false,
		title = undefined,
		onclick = undefined,
		children
	}: {
		variant?: 'primary' | 'ghost' | 'stub';
		size?: 'md' | 'sm';
		href?: string;
		type?: 'button' | 'submit';
		disabled?: boolean;
		title?: string;
		onclick?: (event: MouseEvent) => void;
		children: Snippet;
	} = $props();
</script>

{#if href}
	<a class="btn {variant} {size}" class:disabled {href} {title} {onclick}>
		{@render children()}
	</a>
{:else}
	<button class="btn {variant} {size}" {type} {disabled} {title} {onclick}>
		{@render children()}
	</button>
{/if}

<style>
	.btn {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		border: 1px solid transparent;
		border-radius: var(--radius-sm);
		padding: 0 13px;
		height: 34px;
		font: inherit;
		font-size: 13.5px;
		font-weight: 550;
		line-height: 1;
		color: var(--text);
		background: transparent;
		cursor: pointer;
		text-decoration: none;
		white-space: nowrap;
		transition:
			background 0.12s ease,
			border-color 0.12s ease,
			box-shadow 0.12s ease;
	}
	.btn.sm {
		height: 28px;
		padding: 0 10px;
		font-size: 12.5px;
	}
	.btn.primary {
		background: var(--accent);
		color: var(--accent-fg);
	}
	.btn.primary:hover {
		box-shadow: var(--shadow);
		filter: brightness(1.05);
	}
	.btn.ghost {
		background: var(--surface);
		border-color: var(--border);
	}
	.btn.ghost:hover {
		border-color: var(--border-strong);
		background: var(--surface-2);
	}
	.btn.stub {
		background: var(--stub-soft);
		color: var(--stub);
		border-color: var(--stub-line);
	}
	.btn.stub:hover {
		filter: brightness(0.98);
	}
	.btn:disabled,
	.btn.disabled {
		opacity: 0.5;
		pointer-events: none;
	}
</style>
