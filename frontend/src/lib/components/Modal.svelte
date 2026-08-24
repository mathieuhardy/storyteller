<script lang="ts">
	// Structuring modal (docs/ui/components.md §6): centered dialog with a typed
	// header (kicker + title), a body slot, and a footer slot. Closes on
	// backdrop click or Escape.
	import type { Snippet } from 'svelte';

	let {
		open = false,
		kicker,
		title,
		tone = 'default',
		onclose,
		body,
		footer
	}: {
		open?: boolean;
		kicker: string;
		title: string;
		tone?: 'default' | 'amb';
		onclose: () => void;
		body: Snippet;
		footer?: Snippet;
	} = $props();

	function onKeydown(event: KeyboardEvent) {
		if (open && event.key === 'Escape') onclose();
	}
</script>

<svelte:window onkeydown={onKeydown} />

{#if open}
	<div class="modal-wrap">
		<div class="modal-back" onclick={onclose} role="presentation"></div>
		<div class="modal" role="dialog" aria-modal="true">
			<button class="close" onclick={onclose} aria-label="close">✕</button>
			<div class="mhead" class:amb={tone === 'amb'}>
				<div class="kicker">{kicker}</div>
				<h3>{title}</h3>
			</div>
			<div class="mbody">{@render body()}</div>
			{#if footer}
				<div class="mfoot">{@render footer()}</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.modal-wrap {
		position: fixed;
		inset: 0;
		z-index: 60;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 24px;
	}
	.modal-back {
		position: absolute;
		inset: 0;
		background: rgba(15, 18, 24, 0.45);
	}
	.modal {
		position: relative;
		width: 560px;
		max-width: 100%;
		max-height: 90vh;
		overflow-y: auto;
		background: var(--surface);
		border: 1px solid var(--border-strong);
		border-radius: 14px;
		box-shadow: var(--shadow-pop);
	}
	.close {
		position: absolute;
		top: 14px;
		right: 14px;
		color: var(--faint);
		cursor: pointer;
		width: 26px;
		height: 26px;
		display: grid;
		place-items: center;
		border-radius: 6px;
		border: 0;
		background: transparent;
		font-size: 13px;
	}
	.close:hover {
		background: var(--surface-2);
		color: var(--text);
	}
	.mhead {
		padding: 18px 20px 14px;
		border-bottom: 1px solid var(--border);
	}
	.kicker {
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.07em;
		color: var(--stub);
		font-weight: 600;
	}
	.mhead.amb .kicker {
		color: var(--warning);
	}
	.mhead h3 {
		margin: 6px 0 0;
		font-size: 20px;
		letter-spacing: -0.01em;
	}
	.mbody {
		padding: 18px 20px;
	}
	.mfoot {
		padding: 14px 20px;
		border-top: 1px solid var(--border);
		display: flex;
		align-items: center;
		gap: 10px;
		background: var(--surface-2);
		border-radius: 0 0 14px 14px;
	}
</style>
