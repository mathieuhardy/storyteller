<script lang="ts">
	// Top bar for book mode: logo, filename, word count, save status, settings.
	import Icon from '$components/Icon.svelte';
	import WordCount from './WordCount.svelte';
	import AutoSaveSettings from './AutoSaveSettings.svelte';
	import DisplaySettings from './DisplaySettings.svelte';
	import { t, formatDate } from '$i18n/index.svelte';

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
		onSave,
		onAutoSaveChange,
		onDisplayChange
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
		onSave?: () => void;
		onAutoSaveChange?: (enabled: boolean, interval: number) => void;
		onDisplayChange?: (showLineBreaks: boolean, lineHeight: 'compact' | 'normal' | 'spacious') => void;
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
		onChange={(breaks, height) => onDisplayChange?.(breaks, height)}
	/>
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
</style>
