<script lang="ts">
	// Diagnostic banner (docs/ui/components.md §6): non-blocking alert showing
	// entry errors like missing assets, unresolved links, or YAML issues.
	import type { Diagnostic } from '$api/types';
	import { t } from '$i18n/index.svelte';
	import Icon from '$components/Icon.svelte';

	let { errors }: { errors: Diagnostic[] } = $props();

	const errorCount = $derived(errors.filter((e) => e.severity === 'error').length);
	const warningCount = $derived(errors.filter((e) => e.severity === 'warning').length);
</script>

<div class="diagnostic-banner">
	<div class="header">
		<Icon name="alert-triangle" size={16} />
		<span class="summary">
			{#if errorCount > 0}
				{t('entry.errorCount', { count: errorCount })}
			{/if}
			{#if errorCount > 0 && warningCount > 0}
				{' · '}
			{/if}
			{#if warningCount > 0}
				{t('entry.warningCount', { count: warningCount })}
			{/if}
		</span>
	</div>
	<ul class="list">
		{#each errors as error}
			<li class="item" class:is-error={error.severity === 'error'}>
				<span class="code">{error.code}</span>
				<span class="message">{error.message}</span>
				{#if error.field}
					<span class="field">{error.field}</span>
				{/if}
			</li>
		{/each}
	</ul>
</div>

<style>
	.diagnostic-banner {
		background: var(--danger-soft);
		border: 1px solid var(--danger-line);
		border-radius: var(--radius);
		padding: 12px 14px;
		margin-bottom: 20px;
	}

	.header {
		display: flex;
		align-items: center;
		gap: 8px;
		color: var(--danger);
		font-weight: 600;
		font-size: 13px;
		margin-bottom: 8px;
	}

	.list {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.item {
		display: flex;
		align-items: baseline;
		gap: 8px;
		font-size: 12.5px;
		color: var(--text);
		padding: 4px 0;
	}

	.code {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--danger);
		background: var(--surface);
		padding: 1px 6px;
		border-radius: 3px;
	}

	.item:not(.is-error) .code {
		color: var(--warning);
	}

	.message {
		flex: 1;
	}

	.field {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--muted);
	}
	.field::before {
		content: '@ ';
	}
</style>
