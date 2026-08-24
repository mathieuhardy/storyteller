<script lang="ts">
	// Search screen (docs/api.md §3): results ranked by relevance, each with a
	// highlighted snippet. `q` and pagination live in the URL, so the search
	// box, re-searching and the pager just read/rewrite the query string.
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { t, typeLabel, formatNumber } from '$i18n/index.svelte';
	import Pager from '$components/list/Pager.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	let queryInput = $state(data.q);

	// The URL is the source of truth; keep the input in sync when navigation
	// happens some other way (back/forward, a link elsewhere).
	$effect(() => {
		queryInput = data.q;
	});

	function submitSearch(event: SubmitEvent) {
		event.preventDefault();
		const trimmed = queryInput.trim();
		const params = new URLSearchParams(page.url.searchParams);
		if (trimmed) params.set('q', trimmed);
		else params.delete('q');
		params.delete('page');
		goto(`?${params.toString()}`);
	}

	function setPage(next: number) {
		const params = new URLSearchParams(page.url.searchParams);
		params.set('page', String(next));
		goto(`?${params.toString()}`, { keepFocus: true, noScroll: true });
	}

	function setPerPage(next: number) {
		const params = new URLSearchParams(page.url.searchParams);
		params.set('per_page', String(next));
		params.delete('page');
		goto(`?${params.toString()}`);
	}

	// Focuses the search input on landing, without the `autofocus` attribute
	// (flagged for stealing focus outside the user's control on a page load —
	// here the user just navigated here specifically to type a query).
	function focusOnMount(node: HTMLInputElement) {
		node.focus();
	}
</script>

<div class="screen">
	<form class="searchbar" onsubmit={submitSearch}>
		<input
			class="sinput"
			type="search"
			bind:value={queryInput}
			placeholder={t('search.placeholder')}
			use:focusOnMount
		/>
		<button class="sbtn" type="submit">{t('search.action')}</button>
	</form>

	{#if !data.q.trim()}
		<p class="empty">{t('search.prompt')}</p>
	{:else if data.results.items.length === 0}
		<p class="empty">{t('search.noResults', { query: data.q })}</p>
	{:else}
		<p class="count">{t('search.resultCount', { count: formatNumber(data.results.total), query: data.q })}</p>
		<ul class="results">
			{#each data.results.items as result (result.slug)}
				<li class="result">
					<a class="rlink" href="/entry/{result.slug}">
						<div class="rhead">
							<span class="rtype">{typeLabel(result.type)}</span>
							<span class="rtitle">{result.title}</span>
							{#if result.has_errors}<span class="rflag" title={t('list.hasErrors')}>▲</span>{/if}
						</div>
						<p class="rsnippet">{@html result.snippet}</p>
					</a>
				</li>
			{/each}
		</ul>
		<Pager
			page={data.results.page}
			perPage={data.results.per_page}
			total={data.results.total}
			onpage={setPage}
			onperpage={setPerPage}
		/>
	{/if}
</div>

<style>
	.screen {
		max-width: 720px;
		margin: 0 auto;
		padding: 22px 26px 60px;
	}
	.searchbar {
		display: flex;
		gap: 8px;
		margin-bottom: 20px;
	}
	.sinput {
		flex: 1;
		height: 38px;
		padding: 0 14px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--text);
		font: inherit;
		font-size: 15px;
	}
	.sinput:focus {
		outline: none;
		border-color: var(--accent);
		box-shadow: 0 0 0 3px var(--accent-soft);
	}
	.sbtn {
		height: 38px;
		padding: 0 18px;
		border: 0;
		border-radius: var(--radius-sm);
		background: var(--accent);
		color: var(--accent-fg);
		font: inherit;
		font-size: 13.5px;
		font-weight: 550;
		cursor: pointer;
	}
	.empty,
	.count {
		color: var(--muted);
		font-size: 13.5px;
	}
	.empty {
		padding: 40px 0;
		text-align: center;
	}
	.results {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.result {
		border-bottom: 1px solid var(--border);
	}
	.result:last-child {
		border-bottom: none;
	}
	.rlink {
		display: block;
		padding: 14px 4px;
		text-decoration: none;
		color: inherit;
	}
	.rlink:hover .rtitle {
		color: var(--accent);
	}
	.rhead {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.rtype {
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		padding: 2px 7px;
		border-radius: 4px;
		background: var(--accent-soft);
		color: var(--accent);
	}
	.rtitle {
		font-size: 15px;
		font-weight: 600;
	}
	.rflag {
		color: var(--warning);
		font-size: 11px;
	}
	.rsnippet {
		margin: 6px 0 0;
		font-size: 13px;
		line-height: 1.6;
		color: var(--muted);
	}
	.rsnippet :global(mark) {
		background: var(--accent-soft);
		color: var(--accent);
		border-radius: 2px;
		padding: 0 1px;
	}
</style>
