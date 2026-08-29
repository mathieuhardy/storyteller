<script lang="ts">
	// Auto-save settings popover for book mode.
	import { t } from '$i18n/index.svelte';
	import Popover from '$components/Popover.svelte';
	import Icon from '$components/Icon.svelte';

	let {
		enabled,
		interval,
		onChange
	}: {
		enabled: boolean;
		interval: number;
		onChange: (enabled: boolean, interval: number) => void;
	} = $props();

	let open = $state(false);

	const intervals = [
		{ value: 30_000, label: '30s' },
		{ value: 60_000, label: '1 min' },
		{ value: 120_000, label: '2 min' },
		{ value: 300_000, label: '5 min' }
	];
</script>

<Popover bind:open align="right">
	{#snippet trigger({ toggle })}
		<button class="trigger" class:active={enabled} onclick={toggle} title={t('book.autoSave')}>
			<Icon name="settings" size={14} />
			<span class="label">{t('book.autoSave')}</span>
			{#if enabled}
				<span class="badge mono">{intervals.find((i) => i.value === interval)?.label ?? '1 min'}</span>
			{/if}
		</button>
	{/snippet}

	<div class="panel">
		<p class="panel-label">{t('book.autoSave')}</p>
		<label class="toggle-row">
			<input
				type="checkbox"
				checked={enabled}
				onchange={(e) => onChange(e.currentTarget.checked, interval)}
			/>
			<span>{t('book.autoSaveEnabled')}</span>
		</label>

		{#if enabled}
			<p class="panel-label">{t('book.autoSaveInterval')}</p>
			<div class="interval-options">
				{#each intervals as { value, label } (value)}
					<button
						class="interval-btn"
						class:active={interval === value}
						onclick={() => onChange(enabled, value)}
					>
						{label}
					</button>
				{/each}
			</div>
		{/if}
	</div>
</Popover>

<style>
	.trigger {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 8px;
		border: 0;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--muted);
		font: inherit;
		font-size: 12px;
		cursor: pointer;
	}
	.trigger:hover {
		background: var(--surface-2);
		color: var(--text);
	}
	.trigger.active {
		color: var(--accent);
	}
	.label {
		display: none;
	}
	@media (min-width: 640px) {
		.label {
			display: inline;
		}
	}
	.badge {
		padding: 1px 5px;
		border-radius: 4px;
		background: var(--accent-soft);
		color: var(--accent);
		font-size: 10px;
	}
	.panel {
		padding: 8px;
		min-width: 180px;
	}
	.panel-label {
		margin: 0 0 6px;
		padding: 0 4px;
		font-size: 10.5px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--faint);
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
	.interval-options {
		display: flex;
		gap: 4px;
		margin-top: 4px;
	}
	.interval-btn {
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
	.interval-btn:hover {
		background: var(--surface-2);
	}
	.interval-btn.active {
		background: var(--accent-soft);
		border-color: var(--accent);
		color: var(--accent);
	}
</style>
