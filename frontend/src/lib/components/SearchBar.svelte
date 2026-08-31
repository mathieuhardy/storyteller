<script lang="ts">
	// In-editor search bar. Search triggers on Enter only.
	// Shows on Ctrl+F, hides on Escape.
	import { t } from '$i18n/index.svelte';
	import Icon from './Icon.svelte';

	let {
		content = '',
		visible = $bindable(false),
		onNavigate = (_index: number, _start: number, _end: number) => {}
	}: {
		content?: string;
		visible?: boolean;
		onNavigate?: (index: number, start: number, end: number) => void;
	} = $props();

	let query = $state('');
	let inputEl: HTMLInputElement | undefined = $state();
	let matches: { start: number; end: number }[] = $state([]);
	let currentIndex = $state(0);
	let hasSearched = $state(false);

	// Focus input when search bar becomes visible
	$effect(() => {
		if (visible && inputEl) {
			inputEl.focus();
			inputEl.select();
		}
	});

	// Reset when query changes (user is typing new search)
	$effect(() => {
		query; // Subscribe to query changes
		hasSearched = false;
	});

	function findMatches() {
		if (!query || query.length === 0) {
			matches = [];
			return;
		}
		const results: { start: number; end: number }[] = [];
		const lowerContent = content.toLowerCase();
		const lowerQuery = query.toLowerCase();
		let pos = 0;
		while (true) {
			const idx = lowerContent.indexOf(lowerQuery, pos);
			if (idx === -1) break;
			results.push({ start: idx, end: idx + query.length });
			pos = idx + 1;
		}
		matches = results;
	}

	function search() {
		if (!hasSearched) {
			// First Enter: find matches and go to first
			findMatches();
			hasSearched = true;
			currentIndex = 0;
			if (matches.length > 0) {
				const match = matches[0];
				onNavigate(0, match.start, match.end);
			}
		} else {
			// Subsequent Enter: go to next
			goToNext();
		}
	}

	function goToNext() {
		if (matches.length === 0) return;
		currentIndex = (currentIndex + 1) % matches.length;
		const match = matches[currentIndex];
		onNavigate(currentIndex, match.start, match.end);
	}

	function goToPrev() {
		if (matches.length === 0) return;
		currentIndex = (currentIndex - 1 + matches.length) % matches.length;
		const match = matches[currentIndex];
		onNavigate(currentIndex, match.start, match.end);
	}

	function close() {
		visible = false;
		query = '';
		matches = [];
		currentIndex = 0;
		hasSearched = false;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			close();
		} else if (e.key === 'Enter') {
			e.preventDefault();
			if (e.shiftKey) {
				if (hasSearched) goToPrev();
				else search();
			} else {
				search();
			}
		} else if (e.key === 'F3') {
			e.preventDefault();
			if (!hasSearched) search();
			else if (e.shiftKey) goToPrev();
			else goToNext();
		}
	}
</script>

{#if visible}
	<div class="search-bar" role="search">
		<div class="search-input-wrap">
			<Icon name="search" size={14} />
			<input
				type="text"
				bind:this={inputEl}
				bind:value={query}
				onkeydown={handleKeydown}
				placeholder={t('editorSearch.placeholder')}
				spellcheck="false"
			/>
		</div>

		<div class="search-info">
			{#if hasSearched}
				{#if matches.length === 0}
					<span class="no-results">{t('editorSearch.noResults')}</span>
				{:else}
					<span class="count">{currentIndex + 1} / {matches.length}</span>
				{/if}
			{/if}
		</div>

		<div class="search-nav">
			<button
				type="button"
				class="nav-btn"
				onclick={goToPrev}
				disabled={!hasSearched || matches.length === 0}
				title={t('editorSearch.previous')}
			>
				<Icon name="chevron-up" size={14} />
			</button>
			<button
				type="button"
				class="nav-btn"
				onclick={goToNext}
				disabled={!hasSearched || matches.length === 0}
				title={t('editorSearch.next')}
			>
				<Icon name="chevron-down" size={14} />
			</button>
		</div>

		<button type="button" class="close-btn" onclick={close} title={t('editorSearch.close')}>
			<Icon name="x" size={14} />
		</button>
	</div>
{/if}

<style>
	.search-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 12px;
		background: var(--surface);
		border-bottom: 1px solid var(--border);
		font-size: 13px;
	}

	.search-input-wrap {
		display: flex;
		align-items: center;
		gap: 6px;
		flex: 1;
		max-width: 300px;
		padding: 4px 8px;
		background: var(--surface-2);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		color: var(--muted);
	}

	.search-input-wrap:focus-within {
		border-color: var(--accent);
		box-shadow: 0 0 0 2px var(--accent-soft);
	}

	input {
		flex: 1;
		border: 0;
		background: transparent;
		color: var(--text);
		font-size: 13px;
		outline: none;
		min-width: 0;
	}

	input::placeholder {
		color: var(--faint);
	}

	.search-info {
		min-width: 60px;
		text-align: center;
	}

	.count {
		color: var(--muted);
		font-variant-numeric: tabular-nums;
	}

	.no-results {
		color: var(--warning);
	}

	.search-nav {
		display: flex;
		gap: 2px;
	}

	.nav-btn,
	.close-btn {
		width: 26px;
		height: 26px;
		display: grid;
		place-items: center;
		border: 0;
		background: transparent;
		color: var(--muted);
		border-radius: var(--radius-sm);
		cursor: pointer;
	}

	.nav-btn:hover:not(:disabled),
	.close-btn:hover {
		background: var(--surface-2);
		color: var(--text);
	}

	.nav-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>
