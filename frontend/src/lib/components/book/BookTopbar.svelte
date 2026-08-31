<script lang="ts">
	// Top bar for book mode: logo, filename, word count, save status, settings.
	import Icon from '$components/Icon.svelte';
	import WordCount from './WordCount.svelte';
	import AutoSaveSettings from './AutoSaveSettings.svelte';
	import DisplaySettings from './DisplaySettings.svelte';
	import ReplacementsSettings from './ReplacementsSettings.svelte';
	import LanguageToolSettings from './LanguageToolSettings.svelte';
	import { t, formatDate } from '$i18n/index.svelte';
	import type { ReplacementRule, LTConfig } from '$api/client';

	let {
		fileName = null,
		wordCount = 0,
		isDirty = false,
		isSaving = false,
		lastSaved = null,
		autoSaveEnabled = false,
		autoSaveInterval = 60_000,
		showLineBreaks = false,
		lineHeight = 'normal' as 'compact' | 'normal' | 'spacious',
		editorWidth = 'full' as 'medium' | 'wide' | 'full',
		replacementRules = [],
		replacementsError = null,
		ltConfig = { server_url: 'http://localhost:8081', language: 'fr' },
		ltConnected = null,
		ltTesting = false,
		ltError = null,
		ltDrawerOpen = false,
		onSave,
		onAutoSaveChange,
		onDisplayChange,
		onReplacementsChange,
		onLTConfigChange,
		onLTTest,
		onLTDrawerToggle
	}: {
		fileName?: string | null;
		wordCount?: number;
		isDirty?: boolean;
		isSaving?: boolean;
		lastSaved?: Date | null;
		autoSaveEnabled?: boolean;
		autoSaveInterval?: number;
		showLineBreaks?: boolean;
		lineHeight?: 'compact' | 'normal' | 'spacious';
		editorWidth?: 'medium' | 'wide' | 'full';
		replacementRules?: ReplacementRule[];
		replacementsError?: string | null;
		ltConfig?: LTConfig;
		ltConnected?: boolean | null;
		ltTesting?: boolean;
		ltError?: string | null;
		ltDrawerOpen?: boolean;
		onSave?: () => void;
		onAutoSaveChange?: (enabled: boolean, interval: number) => void;
		onDisplayChange?: (showLineBreaks: boolean, lineHeight: 'compact' | 'normal' | 'spacious', editorWidth: 'medium' | 'wide' | 'full') => void;
		onReplacementsChange?: (rules: ReplacementRule[]) => void;
		onLTConfigChange?: (config: LTConfig) => void;
		onLTTest?: () => void;
		onLTDrawerToggle?: () => void;
	} = $props();

	const displayName = $derived(fileName ? fileName.split('/').pop() : null);
</script>

<header class="topbar">
	<a class="brand" href="/">
		<Icon name="book-open" size={18} />
		<span class="brand-name">{t('book.title')}</span>
	</a>

	<div class="file-info">
		{#if displayName}
			<span class="filename">{displayName}</span>
			{#if isDirty}
				<span class="unsaved-dot" title={t('editor.unsaved')}></span>
			{/if}
		{:else}
			<span class="no-file">{t('book.noFileSelected')}</span>
		{/if}
	</div>

	<div class="spacer"></div>

	<WordCount count={wordCount} />

	<div class="divider"></div>

	<div class="save-status">
		{#if isSaving}
			<span class="status saving">{t('editor.saving')}</span>
		{:else if lastSaved}
			<span class="status saved" title={formatDate(lastSaved.toISOString())}>
				{t('book.saved')}
			</span>
		{:else if isDirty}
			<span class="status unsaved">{t('editor.unsaved')}</span>
		{/if}
	</div>

	<button
		class="save-btn"
		disabled={!isDirty || isSaving || !fileName}
		onclick={() => onSave?.()}
		title="Ctrl+S"
	>
		{t('action.save')}
	</button>

	<div class="divider"></div>

	<AutoSaveSettings
		enabled={autoSaveEnabled}
		interval={autoSaveInterval}
		onChange={(enabled, interval) => onAutoSaveChange?.(enabled, interval)}
	/>

	<DisplaySettings
		{showLineBreaks}
		{lineHeight}
		{editorWidth}
		onChange={(breaks, height, width) => onDisplayChange?.(breaks, height, width)}
	/>

	<ReplacementsSettings
		rules={replacementRules}
		error={replacementsError}
		onChange={(rules) => onReplacementsChange?.(rules)}
	/>

	<LanguageToolSettings
		config={ltConfig}
		isConnected={ltConnected}
		isTesting={ltTesting}
		error={ltError}
		onChange={(config) => onLTConfigChange?.(config)}
		onTest={() => onLTTest?.()}
	/>

	<button
		class="drawer-toggle"
		class:active={ltDrawerOpen}
		onclick={() => onLTDrawerToggle?.()}
		title={t('book.ltDrawerTitle')}
	>
		<Icon name="spell-check" size={16} />
	</button>
</header>

<style>
	.topbar {
		display: flex;
		align-items: center;
		gap: 10px;
		height: 48px;
		padding: 0 12px;
		background: var(--surface);
		border-bottom: 1px solid var(--border);
		position: relative;
		z-index: 20;
	}

	.brand {
		display: flex;
		align-items: center;
		gap: 7px;
		color: var(--accent);
		text-decoration: none;
		font-weight: 700;
	}

	.brand-name {
		color: var(--text);
		font-size: 15px;
	}

	.file-info {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.filename {
		font-size: 14px;
		font-weight: 500;
		color: var(--text);
	}

	.no-file {
		font-size: 14px;
		color: var(--muted);
		font-style: italic;
	}

	.unsaved-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--warning);
	}

	.spacer {
		flex: 1;
	}

	.divider {
		width: 1px;
		height: 20px;
		background: var(--border);
	}

	.save-status {
		min-width: 60px;
		text-align: right;
	}

	.status {
		font-size: 11px;
	}

	.status.saving {
		color: var(--muted);
	}

	.status.saved {
		color: var(--ok);
	}

	.status.unsaved {
		color: var(--warning);
	}

	.save-btn {
		padding: 5px 12px;
		border: 0;
		border-radius: var(--radius-sm);
		background: var(--accent);
		color: var(--accent-fg);
		font: inherit;
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
	}

	.save-btn:hover:not(:disabled) {
		opacity: 0.9;
	}

	.save-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.drawer-toggle {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--muted);
		cursor: pointer;
		margin-left: 4px;
	}

	.drawer-toggle:hover {
		background: var(--surface-2);
		color: var(--text);
	}

	.drawer-toggle.active {
		background: var(--accent);
		border-color: var(--accent);
		color: var(--accent-fg);
	}
</style>
