<script lang="ts">
	// Small anchored popover (docs/ui/components.md §6): a trigger and a panel
	// docked under it, closing on an outside click or Escape. Used by the sort
	// and filter menus of the list/table screen.
	import type { Snippet } from 'svelte';

	let {
		open = $bindable(false),
		align = 'left',
		trigger,
		children
	}: {
		open?: boolean;
		align?: 'left' | 'right';
		trigger: Snippet<[{ toggle: () => void }]>;
		children: Snippet;
	} = $props();

	let root: HTMLDivElement | undefined = $state();

	function toggle() {
		open = !open;
	}

	function onWindowClick(event: MouseEvent) {
		if (open && root && !root.contains(event.target as Node)) open = false;
	}

	function onWindowKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') open = false;
	}
</script>

<svelte:window onclick={onWindowClick} onkeydown={onWindowKeydown} />

<div class="popover-root" bind:this={root}>
	{@render trigger({ toggle })}
	{#if open}
		<div class="popover-panel" class:right={align === 'right'}>
			{@render children()}
		</div>
	{/if}
</div>

<style>
	.popover-root {
		position: relative;
		display: inline-flex;
	}
	.popover-panel {
		position: absolute;
		top: calc(100% + 6px);
		left: 0;
		z-index: 30;
		min-width: 200px;
		background: var(--surface);
		border: 1px solid var(--border-strong);
		border-radius: var(--radius);
		box-shadow: var(--shadow-pop);
		padding: 5px;
	}
	.popover-panel.right {
		left: auto;
		right: 0;
	}
</style>
