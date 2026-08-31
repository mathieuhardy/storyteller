<script lang="ts">
	// LanguageTool configuration popover for book mode.
	// Allows configuring server URL and language for grammar checking.
	import { t } from '$i18n/index.svelte';
	import Popover from '$components/Popover.svelte';
	import Icon from '$components/Icon.svelte';
	import type { LTConfig } from '$api/client';

	let {
		config,
		isConnected = null,
		isTesting = false,
		error = null,
		onChange,
		onTest
	}: {
		config: LTConfig;
		isConnected?: boolean | null;
		isTesting?: boolean;
		error?: string | null;
		onChange: (config: LTConfig) => void;
		onTest: () => void;
	} = $props();

	let open = $state(false);

	function updateField(field: keyof LTConfig, value: string) {
		onChange({ ...config, [field]: value });
	}
</script>

<Popover bind:open align="right">
	{#snippet trigger({ toggle })}
		<button
			class="trigger"
			class:connected={isConnected === true}
			class:disconnected={isConnected === false}
			onclick={toggle}
			title={t('book.languageTool')}
		>
			<Icon name="spell-check" size={14} />
		</button>
	{/snippet}

	<div class="panel">
		<p class="panel-label">{t('book.languageTool')}</p>
		<p class="hint">{t('book.languageToolHint')}</p>

		<div class="field">
			<label for="lt-server">{t('book.ltServerUrl')}</label>
			<input
				id="lt-server"
				type="url"
				value={config.server_url}
				placeholder="http://localhost:8081"
				onchange={(e) => updateField('server_url', e.currentTarget.value)}
			/>
		</div>

		<div class="field">
			<label for="lt-language">{t('book.ltLanguage')}</label>
			<input
				id="lt-language"
				type="text"
				value={config.language}
				placeholder="fr"
				onchange={(e) => updateField('language', e.currentTarget.value)}
			/>
		</div>

		<div class="actions">
			<button class="test-btn" onclick={onTest} disabled={isTesting}>
				{#if isTesting}
					{t('book.ltTesting')}
				{:else}
					{t('book.ltTestConnection')}
				{/if}
			</button>

			{#if isConnected === true}
				<span class="status ok">
					<Icon name="check" size={12} />
					{t('book.ltConnected')}
				</span>
			{:else if isConnected === false}
				<span class="status error">
					<Icon name="x" size={12} />
					{t('book.ltDisconnected')}
				</span>
			{/if}
		</div>

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
	.trigger.connected {
		color: var(--ok);
	}
	.trigger.disconnected {
		color: var(--warning);
	}
	.panel {
		padding: 8px;
		min-width: 280px;
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
		margin: 0 0 10px;
		padding: 0 4px;
		font-size: 11px;
		color: var(--faint);
	}
	.field {
		margin-bottom: 8px;
	}
	.field label {
		display: block;
		margin-bottom: 3px;
		padding: 0 4px;
		font-size: 11px;
		color: var(--muted);
	}
	.field input {
		width: 100%;
		padding: 6px 8px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--text);
		font: inherit;
		font-size: 12px;
	}
	.field input:focus {
		outline: none;
		border-color: var(--accent);
	}
	.actions {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-top: 10px;
	}
	.test-btn {
		padding: 5px 10px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--surface-2);
		color: var(--text);
		font: inherit;
		font-size: 11px;
		cursor: pointer;
	}
	.test-btn:hover:not(:disabled) {
		background: var(--surface-3);
	}
	.test-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.status {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
	}
	.status.ok {
		color: var(--ok);
	}
	.status.error {
		color: var(--warning);
	}
	.error {
		margin: 8px 0 0;
		padding: 0 4px;
		font-size: 11px;
		color: var(--warning);
	}
</style>
