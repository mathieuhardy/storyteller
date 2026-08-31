<script lang="ts">
	// Book mode: raw markdown editor with file tree sidebar.
	// Works standalone — no Storyteller project required.
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import BookTopbar from '$components/book/BookTopbar.svelte';
	import FileTree from '$components/book/FileTree.svelte';
	import RawEditor from '$components/book/RawEditor.svelte';
	import LanguageToolDrawer from '$components/book/LanguageToolDrawer.svelte';
	import {
		listBookFiles,
		readBookFile,
		writeBookFile,
		getBookReplacements,
		setBookReplacements,
		getLTConfig,
		setLTConfig,
		testLTConnection,
		checkLT
	} from '$api/client';
	import type { ReplacementRule, LTConfig, LTMatch } from '$api/client';

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
	let editorWidth = $state<'medium' | 'wide' | 'full'>('full');

	// Folders to show in file tree (persisted to localStorage)
	let visibleFolders = $state<string[]>(['chapters']);

	// Post-save replacement rules (persisted server-side, .storyteller/replacements.yaml)
	let replacementRules = $state<ReplacementRule[]>([]);
	let replacementsError = $state<string | null>(null);

	// LanguageTool config (persisted server-side, .storyteller/languagetool.yaml)
	let ltConfig = $state<LTConfig>({ server_url: 'http://localhost:8081', language: 'fr' });
	let ltConnected = $state<boolean | null>(null);
	let ltTesting = $state(false);
	let ltError = $state<string | null>(null);

	// LanguageTool drawer state
	let ltDrawerOpen = $state(false);
	let ltMatches = $state<LTMatch[]>([]);
	let ltChecking = $state(false);

	onMount(async () => {
		try {
			const res = await getBookReplacements();
			replacementRules = res.rules;
		} catch (err) {
			console.error('Failed to load replacement rules:', err);
		}

		try {
			const res = await getLTConfig();
			ltConfig = { server_url: res.server_url, language: res.language };
		} catch (err) {
			console.error('Failed to load LanguageTool config:', err);
		}
	});

	async function updateReplacementRules(rules: ReplacementRule[]) {
		const previous = replacementRules;
		// Show every row immediately, including a freshly-added one still
		// missing its `find` text — but only persist the rules that are
		// actually complete (the backend rejects an empty `find`), so typing
		// into a new row's fields doesn't get wiped out by a failed save.
		replacementRules = rules;
		try {
			await setBookReplacements(rules.filter((rule) => rule.find !== ''));
			replacementsError = null;
		} catch (err) {
			replacementRules = previous;
			replacementsError = err instanceof Error ? err.message : String(err);
			console.error('Failed to save replacement rules:', err);
		}
	}

	async function updateLTConfig(config: LTConfig) {
		const previous = ltConfig;
		ltConfig = config;
		ltConnected = null; // Reset connection status on config change
		try {
			await setLTConfig(config);
			ltError = null;
		} catch (err) {
			ltConfig = previous;
			ltError = err instanceof Error ? err.message : String(err);
			console.error('Failed to save LanguageTool config:', err);
		}
	}

	async function testLT() {
		ltTesting = true;
		ltError = null;
		try {
			await testLTConnection();
			ltConnected = true;
		} catch (err) {
			ltConnected = false;
			ltError = err instanceof Error ? err.message : String(err);
			console.error('LanguageTool connection test failed:', err);
		} finally {
			ltTesting = false;
		}
	}

	async function checkDocument() {
		if (!content) return;
		ltChecking = true;
		ltMatches = [];
		try {
			const result = await checkLT(content, ltConfig.language);
			ltMatches = result.matches;
			ltConnected = true;
		} catch (err) {
			ltConnected = false;
			console.error('LanguageTool check failed:', err);
		} finally {
			ltChecking = false;
		}
	}

	function applyFix(match: LTMatch, replacement: string) {
		// Replace the text at the match offset with the replacement
		const before = content.slice(0, match.offset);
		const after = content.slice(match.offset + match.length);
		content = before + replacement + after;

		// Remove this match and adjust offsets of subsequent matches
		const delta = replacement.length - match.length;
		ltMatches = ltMatches
			.filter((m) => m !== match)
			.map((m) => {
				if (m.offset > match.offset) {
					return { ...m, offset: m.offset + delta };
				}
				return m;
			});
	}

	function toggleLTDrawer() {
		ltDrawerOpen = !ltDrawerOpen;
		if (ltDrawerOpen && ltMatches.length === 0 && content && !ltChecking) {
			checkDocument();
		}
	}

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
			editorWidth = parsed.editorWidth ?? 'full';
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
				JSON.stringify({ showLineBreaks, lineHeight, editorWidth })
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
			const file = await readBookFile(path);
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
			const saved = await writeBookFile(currentFile, content);
			content = saved.content;
			originalContent = saved.content;
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

<div class="topbar-row">
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
		{editorWidth}
		{replacementRules}
		{replacementsError}
		{ltConfig}
		{ltConnected}
		{ltTesting}
		{ltError}
		onSave={save}
		onAutoSaveChange={(enabled, interval) => {
			autoSaveEnabled = enabled;
			autoSaveInterval = interval;
		}}
		onDisplayChange={(breaks, height, width) => {
			showLineBreaks = breaks;
			lineHeight = height;
			editorWidth = width;
		}}
		onReplacementsChange={updateReplacementRules}
		onLTConfigChange={updateLTConfig}
		onLTTest={testLT}
		{ltDrawerOpen}
		onLTDrawerToggle={toggleLTDrawer}
	/>
</div>

<aside class="sidebar">
	<FileTree
		folders={visibleFolders}
		{currentFile}
		onSelect={selectFile}
		onFoldersChange={(folders) => (visibleFolders = folders)}
	/>
</aside>

<div class="editor">
	<RawEditor
		bind:content
		{showLineBreaks}
		{lineHeight}
		{editorWidth}
		disabled={!currentFile}
		placeholder={currentFile ? '' : 'Select a file from the sidebar to start editing.'}
	/>
</div>

<LanguageToolDrawer
	open={ltDrawerOpen}
	matches={ltMatches}
	isChecking={ltChecking}
	onClose={() => (ltDrawerOpen = false)}
	onCheck={checkDocument}
	onApplyFix={applyFix}
/>

<style>
	.topbar-row {
		grid-column: 1 / -1;
	}

	.sidebar {
		display: flex;
		flex-direction: column;
		min-width: 0;
		min-height: 0;
		background: var(--surface);
		border-right: 1px solid var(--border);
	}

	.editor {
		display: flex;
		flex-direction: column;
		min-width: 0;
		min-height: 0;
	}

	@media (max-width: 768px) {
		.sidebar {
			display: none;
		}
	}
</style>
