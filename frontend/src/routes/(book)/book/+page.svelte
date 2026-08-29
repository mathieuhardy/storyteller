<script lang="ts">
	// Book mode: raw markdown editor with file tree sidebar.
	import { browser } from '$app/environment';
	import BookTopbar from '$components/book/BookTopbar.svelte';
	import FileTree from '$components/book/FileTree.svelte';
	import RawEditor from '$components/book/RawEditor.svelte';
	import { listFiles, readFile, writeFile } from '$api/client';
	import type { LayoutData } from '../$types';

	let { data }: { data: LayoutData } = $props();

	// State
	let currentFile = $state<string | null>(null);
	let content = $state('');
	let originalContent = $state('');
	let isDirty = $derived(content !== originalContent);
	let isSaving = $state(false);
	let lastSaved = $state<Date | null>(null);

	// Auto-save settings (persisted to localStorage)
	let autoSaveEnabled = $state(false);
	let autoSaveInterval = $state(60_000); // 1 minute default
	let autoSaveTimer: ReturnType<typeof setInterval> | null = null;

	// Display settings (persisted to localStorage)
	let showLineBreaks = $state(false);
	let lineHeight = $state<'compact' | 'normal' | 'spacious'>('normal');

	// Folders to show in file tree (persisted to localStorage)
	let visibleFolders = $state<string[]>(['chapters']);

	// Initialize from localStorage
	if (browser) {
		const savedAutoSave = localStorage.getItem('book-mode-auto-save');
		if (savedAutoSave) {
			const parsed = JSON.parse(savedAutoSave);
			autoSaveEnabled = parsed.enabled ?? false;
			autoSaveInterval = parsed.interval ?? 60_000;
		}

		const savedDisplay = localStorage.getItem('book-mode-display');
		if (savedDisplay) {
			const parsed = JSON.parse(savedDisplay);
			showLineBreaks = parsed.showLineBreaks ?? false;
			lineHeight = parsed.lineHeight ?? 'normal';
		}

		const savedFolders = localStorage.getItem('book-mode-folders');
		if (savedFolders) {
			visibleFolders = JSON.parse(savedFolders);
		}
	}

	// Persist settings changes
	$effect(() => {
		if (browser) {
			localStorage.setItem(
				'book-mode-auto-save',
				JSON.stringify({ enabled: autoSaveEnabled, interval: autoSaveInterval })
			);
		}
	});

	$effect(() => {
		if (browser) {
			localStorage.setItem(
				'book-mode-display',
				JSON.stringify({ showLineBreaks, lineHeight })
			);
		}
	});

	$effect(() => {
		if (browser) {
			localStorage.setItem('book-mode-folders', JSON.stringify(visibleFolders));
		}
	});

	// Auto-save effect
	$effect(() => {
		if (autoSaveTimer) {
			clearInterval(autoSaveTimer);
			autoSaveTimer = null;
		}

		if (autoSaveEnabled && isDirty && currentFile) {
			autoSaveTimer = setInterval(() => {
				if (isDirty && currentFile) {
					save();
				}
			}, autoSaveInterval);
		}

		return () => {
			if (autoSaveTimer) {
				clearInterval(autoSaveTimer);
			}
		};
	});

	// Keyboard shortcuts
	function handleKeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key === 's') {
			e.preventDefault();
			if (isDirty && currentFile) {
				save();
			}
		}
	}

	async function selectFile(path: string) {
		// Warn if there are unsaved changes
		if (isDirty && currentFile) {
			const confirmed = confirm('You have unsaved changes. Discard them?');
			if (!confirmed) return;
		}

		try {
			const file = await readFile(path);
			currentFile = path;
			content = file.content;
			originalContent = file.content;
			lastSaved = null;
		} catch (err) {
			console.error('Failed to load file:', err);
		}
	}

	async function save() {
		if (!currentFile || !isDirty) return;

		isSaving = true;
		try {
			await writeFile(currentFile, content);
			originalContent = content;
			lastSaved = new Date();
		} catch (err) {
			console.error('Failed to save file:', err);
		} finally {
			isSaving = false;
		}
	}

	// Word count
	const wordCount = $derived.by(() => {
		if (!content) return 0;
		return content
			.trim()
			.split(/\s+/)
			.filter((w) => w.length > 0).length;
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<aside class="sidebar">
	<FileTree
		folders={visibleFolders}
		{currentFile}
		onSelect={selectFile}
		onFoldersChange={(folders) => (visibleFolders = folders)}
	/>
</aside>

<div class="main">
	<BookTopbar
		fileName={currentFile}
		{wordCount}
		{isDirty}
		{isSaving}
		{lastSaved}
		{autoSaveEnabled}
		{autoSaveInterval}
		{showLineBreaks}
		{lineHeight}
		onSave={save}
		onAutoSaveChange={(enabled, interval) => {
			autoSaveEnabled = enabled;
			autoSaveInterval = interval;
		}}
		onDisplayChange={(breaks, height) => {
			showLineBreaks = breaks;
			lineHeight = height;
		}}
	/>
	<RawEditor
		bind:content
		{showLineBreaks}
		{lineHeight}
		disabled={!currentFile}
		placeholder={currentFile ? '' : 'Select a file from the sidebar to start editing.'}
	/>
</div>

<style>
	.sidebar {
		display: flex;
		flex-direction: column;
		min-width: 0;
		overflow: hidden;
		background: var(--book-surface, var(--surface));
		border-right: 1px solid var(--book-border, var(--border));
	}

	.main {
		display: flex;
		flex-direction: column;
		min-width: 0;
		overflow: hidden;
	}

	@media (max-width: 768px) {
		.sidebar {
			display: none;
		}
	}
</style>
