<script lang="ts">
	// Post-save replacement rules popover for book mode (find/replace pairs
	// applied server-side on save, e.g. -- -> —). Presentational: state lives
	// in +page.svelte, persisted server-side in .storyteller/replacements.yaml.
	import { t } from '$i18n/index.svelte';
	import Popover from '$components/Popover.svelte';
	import Icon from '$components/Icon.svelte';
	import type { ReplacementRule } from '$api/client';

	let {
		rules,
		error = null,
		onChange
	}: {
		rules: ReplacementRule[];
		error?: string | null;
		onChange: (rules: ReplacementRule[]) => void;
	} = $props();

	let open = $state(false);

	function updateRule(index: number, field: 'find' | 'replace', value: string) {
		onChange(rules.map((rule, i) => (i === index ? { ...rule, [field]: value } : rule)));
	}

	function removeRule(index: number) {
		onChange(rules.filter((_, i) => i !== index));
	}

	function addRule() {
		onChange([...rules, { find: '', replace: '' }]);
	}
</script>

<Popover bind:open align="right">
	{#snippet trigger({ toggle })}
		<button class="trigger" onclick={toggle} title={t('book.replacements')}>
			<Icon name="edit" size={14} />
		</button>
	{/snippet}

	<div class="panel">
		<p class="panel-label">{t('book.replacements')}</p>
		<p class="hint">{t('book.replacementsHint')}</p>

		{#each rules as rule, i (i)}
			<div class="rule-row">
				<input
					class="mono"
					value={rule.find}
					placeholder={t('book.replacementsFind')}
					onchange={(e) => updateRule(i, 'find', e.currentTarget.value)}
				/>
				<input
					class="mono"
					value={rule.replace}
					placeholder={t('book.replacementsReplace')}
					onchange={(e) => updateRule(i, 'replace', e.currentTarget.value)}
				/>
				<button
					class="remove-btn"
					onclick={() => removeRule(i)}
					title={t('book.replacementsRemove')}
				>
					<Icon name="x" size={12} />
				</button>
			</div>
		{/each}

		{#if rules.length === 0}
			<p class="empty">{t('book.replacementsEmpty')}</p>
		{/if}

		<button class="add-btn" onclick={addRule}>
			<Icon name="plus" size={12} />
			{t('book.replacementsAdd')}
		</button>

		{#if error}
			<p class="error">{error}</p>
		{/if}
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
		min-width: 260px;
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
	.hint {
		margin: 0 0 8px;
		padding: 0 4px;
		font-size: 11px;
		color: var(--faint);
	}
	.rule-row {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-bottom: 4px;
	}
	.rule-row input {
		flex: 1;
		min-width: 0;
		padding: 4px 6px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--text);
		font: inherit;
		font-size: 12px;
	}
	.remove-btn {
		display: flex;
		align-items: center;
		padding: 2px;
		border: 0;
		background: transparent;
		color: var(--muted);
		cursor: pointer;
	}
	.remove-btn:hover {
		color: var(--text);
	}
	.add-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-top: 4px;
		padding: 6px 8px;
		border: 1px dashed var(--border);
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--muted);
		font: inherit;
		font-size: 12px;
		cursor: pointer;
	}
	.add-btn:hover {
		background: var(--surface-2);
		color: var(--text);
	}
	.empty {
		margin: 4px 0;
		padding: 0 4px;
		font-size: 12px;
		color: var(--muted);
	}
	.error {
		margin: 6px 0 0;
		padding: 0 4px;
		font-size: 11px;
		color: var(--warning);
	}
</style>
