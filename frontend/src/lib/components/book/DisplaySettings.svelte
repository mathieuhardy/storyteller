<script lang="ts">
	// Display settings popover for book mode (line breaks, line height, editor width).
	import { t } from '$i18n/index.svelte';
	import Popover from '$components/Popover.svelte';
	import Icon from '$components/Icon.svelte';

	let {
		showLineBreaks,
		lineHeight,
		editorWidth,
		onChange
	}: {
		showLineBreaks: boolean;
		lineHeight: 'compact' | 'normal' | 'spacious';
		editorWidth: 'medium' | 'wide' | 'full';
		onChange: (
			showLineBreaks: boolean,
			lineHeight: 'compact' | 'normal' | 'spacious',
			editorWidth: 'medium' | 'wide' | 'full'
		) => void;
	} = $props();

	let open = $state(false);

	const lineHeights = [
		{ value: 'compact' as const, label: t('book.lineHeightCompact') },
		{ value: 'normal' as const, label: t('book.lineHeightNormal') },
		{ value: 'spacious' as const, label: t('book.lineHeightSpacious') }
	];

	const editorWidths = [
		{ value: 'medium' as const, label: t('book.widthMedium') },
		{ value: 'wide' as const, label: t('book.widthWide') },
		{ value: 'full' as const, label: t('book.widthFull') }
	];
</script>

<Popover bind:open align="right">
	{#snippet trigger({ toggle })}
		<button class="trigger" onclick={toggle} title={t('book.displaySettings')}>
			<Icon name="monitor" size={14} />
		</button>
	{/snippet}

	<div class="panel">
		<p class="panel-label">{t('book.editorWidth')}</p>
		<div class="options">
			{#each editorWidths as { value, label } (value)}
				<button
					class="option-btn"
					class:active={editorWidth === value}
					onclick={() => onChange(showLineBreaks, lineHeight, value)}
				>
					{label}
				</button>
			{/each}
		</div>

		<p class="panel-label">{t('book.lineHeight')}</p>
		<div class="options">
			{#each lineHeights as { value, label } (value)}
				<button
					class="option-btn"
					class:active={lineHeight === value}
					onclick={() => onChange(showLineBreaks, value, editorWidth)}
				>
					{label}
				</button>
			{/each}
		</div>
	</div>
</Popover>

<style>
	.trigger {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: 0;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--muted);
		cursor: pointer;
	}
	.trigger:hover {
		background: var(--surface-2);
		color: var(--text);
	}
	.panel {
		padding: 8px;
		min-width: 180px;
	}
	.panel-label {
		margin: 8px 0 6px;
		padding: 0 4px;
		font-size: 10.5px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--faint);
	}
	.panel-label:first-child {
		margin-top: 0;
	}
	.options {
		display: flex;
		gap: 4px;
		margin-top: 4px;
	}
	.option-btn {
		flex: 1;
		padding: 6px 8px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--text);
		font: inherit;
		font-size: 12px;
		cursor: pointer;
	}
	.option-btn:hover {
		background: var(--surface-2);
	}
	.option-btn.active {
		background: var(--accent-soft);
		border-color: var(--accent);
		color: var(--accent);
	}
</style>
