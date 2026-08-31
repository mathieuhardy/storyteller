<script lang="ts">
	// In-editor search bar with occurrence count and navigation.
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
	let currentIndex = $state(0);

	// Find all matches
	const matches = $derived.by(() => {
		if (!query || query.length === 0) return [];
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
		return results;
	});

	const matchCount = $derived(matches.length);

	// Reset current index when matches change
	$effect(() => {
		if (matches.length > 0 && currentIndex >= matches.length) {
			currentIndex = 0;
		}
	});

	// Navigate to current match when it changes
	$effect(() => {
		if (matches.length > 0 && currentIndex < matches.length) {
			const match = matches[currentIndex];
			onNavigate(currentIndex, match.start, match.end);
		}
	});

	// Focus input when search bar becomes visible
	$effect(() => {
		if (visible && inputEl) {
			inputEl.focus();
			inputEl.select();
		}
	});

	function goToNext() {
		if (matchCount === 0) return;
		currentIndex = (currentIndex + 1) % matchCount;
	}

	function goToPrev() {
		if (matchCount === 0) return;
		currentIndex = (currentIndex - 1 + matchCount) % matchCount;
	}

	function close() {
		visible = false;
		query = '';
		currentIndex = 0;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			close();
		} else if (e.key === 'Enter') {
			e.preventDefault();
			if (e.shiftKey) {
				goToPrev();
			} else {
				goToNext();
			}
		} else if (e.key === 'F3') {
			e.preventDefault();
			if (e.shiftKey) {
				goToPrev();
			} else {
				goToNext();
			}
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
			{#if query.length > 0}
				{#if matchCount === 0}
					<span class="no-results">{t('editorSearch.noResults')}</span>
				{:else}
					<span class="count">{currentIndex + 1} / {matchCount}</span>
				{/if}
			{/if}
		</div>

		<div class="search-nav">
			<button
				type="button"
				class="nav-btn"
				onclick={goToPrev}
				disabled={matchCount === 0}
				title={t('editorSearch.previous')}
			>
				<Icon name="chevron-up" size={14} />
			</button>
			<button
				type="button"
				class="nav-btn"
				onclick={goToNext}
				disabled={matchCount === 0}
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
