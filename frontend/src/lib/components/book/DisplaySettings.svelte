<script lang="ts">
	// Display settings popover for book mode (line breaks, line height).
	import { t } from '$i18n/index.svelte';
	import Popover from '$components/Popover.svelte';
	import Icon from '$components/Icon.svelte';

	let {
		showLineBreaks,
		lineHeight,
		onChange
	}: {
		showLineBreaks: boolean;
		lineHeight: 'compact' | 'normal' | 'spacious';
		onChange: (showLineBreaks: boolean, lineHeight: 'compact' | 'normal' | 'spacious') => void;
	} = $props();

	let open = $state(false);

	const lineHeights = [
		{ value: 'compact' as const, label: t('book.lineHeightCompact') },
		{ value: 'normal' as const, label: t('book.lineHeightNormal') },
		{ value: 'spacious' as const, label: t('book.lineHeightSpacious') }
	];
</script>

<Popover bind:open align="right">
	{#snippet trigger({ toggle })}
		<button class="trigger" onclick={toggle} title={t('book.displaySettings')}>
			<Icon name="monitor" size={14} />
		</button>
	{/snippet}

	<div class="panel">
		<p class="panel-label">{t('book.displaySettings')}</p>

		<label class="toggle-row">
			<input
				type="checkbox"
				checked={showLineBreaks}
				onchange={(e) => onChange(e.currentTarget.checked, lineHeight)}
			/>
			<span>{t('book.showLineBreaks')}</span>
		</label>

		<p class="panel-label">{t('book.lineHeight')}</p>
		<div class="height-options">
			{#each lineHeights as { value, label } (value)}
				<button
					class="height-btn"
					class:active={lineHeight === value}
					onclick={() => onChange(showLineBreaks, value)}
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
	.toggle-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 4px;
		font-size: 13px;
		cursor: pointer;
	}
	.toggle-row input {
		margin: 0;
	}
	.height-options {
		display: flex;
		gap: 4px;
		margin-top: 4px;
	}
	.height-btn {
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
	.height-btn:hover {
		background: var(--surface-2);
	}
	.height-btn.active {
		background: var(--accent-soft);
		border-color: var(--accent);
		color: var(--accent);
	}
</style>
