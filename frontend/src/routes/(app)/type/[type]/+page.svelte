<script lang="ts">
	// List/table screen (docs/ui/screens.md §3): the entries of one type, as a
	// sortable/filterable table or a card list. Sort, filters and pagination are
	// carried entirely in the URL's query string (`docs/api.md` §4), so the
	// toolbar/chips/pager just read and rewrite `page.url.searchParams` — no
	// separate client-side filter state to keep in sync.
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { t, typeLabel, fieldLabel, formatNumber } from '$i18n/index.svelte';
	import Button from '$components/Button.svelte';
	import SortMenu from '$components/list/SortMenu.svelte';
	import FilterMenu from '$components/list/FilterMenu.svelte';
	import FilterChips, { chipsFromParams } from '$components/list/FilterChips.svelte';
	import EntriesTable from '$components/list/EntriesTable.svelte';
	import EntriesCards from '$components/list/EntriesCards.svelte';
	import Pager from '$components/list/Pager.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	let view = $state<'table' | 'list'>('table');

	const RESERVED_PARAMS = new Set(['sort', 'page', 'per_page']);
	const SORTABLE_KINDS = new Set(['text', 'number', 'number-or-text', 'boolean', 'enum']);

	function parseSort(raw: string | null): { field: string; descending: boolean } {
		if (!raw) return { field: 'updated', descending: true };
		return raw.startsWith('-')
			? { field: raw.slice(1), descending: true }
			: { field: raw, descending: false };
	}

	const sort = $derived(parseSort(page.url.searchParams.get('sort')));

	const sortOptions = $derived(
		data.schema.fields
			.filter((f) => SORTABLE_KINDS.has(f.kind) && f.name !== 'type')
			.map((f) => ({ key: f.name, label: fieldLabel(f) }))
	);

	const filterFields = $derived([
		{ name: 'tag', label: t('list.tagField'), kind: 'text' as const },
		...data.schema.fields
			.filter((f) => f.kind !== 'image' && f.kind !== 'image-list' && f.name !== 'type')
			.map((f) => ({ name: f.name, label: fieldLabel(f), kind: f.kind, enum_values: f.enum_values }))
	]);

	const chips = $derived(
		chipsFromParams(page.url.searchParams, (key) => {
			if (key === 'tag') return t('list.tagField');
			const field = data.schema.fields.find((f) => f.name === key);
			return field ? fieldLabel(field) : key;
		})
	);

	function updateParams(mutate: (params: URLSearchParams) => void, resetPage = true) {
		const params = new URLSearchParams(page.url.searchParams);
		mutate(params);
		if (resetPage) params.delete('page');
		const qs = params.toString();
		goto(qs ? `?${qs}` : page.url.pathname, { keepFocus: true, noScroll: true });
	}

	function setSort(field: string, descending: boolean) {
		updateParams((params) => params.set('sort', descending ? `-${field}` : field));
	}

	function toggleColumnSort(field: string) {
		setSort(field, sort.field === field ? !sort.descending : false);
	}

	function addFilter(key: string, value: string) {
		updateParams((params) => {
			if (key === 'tag') params.append(key, value);
			else params.set(key, value);
		});
	}

	function removeFilter(key: string, value: string) {
		updateParams((params) => {
			const remaining = params.getAll(key).filter((v) => v !== value);
			params.delete(key);
			for (const v of remaining) params.append(key, v);
		});
	}

	function clearAllFilters() {
		updateParams((params) => {
			for (const key of [...params.keys()]) {
				if (!RESERVED_PARAMS.has(key)) params.delete(key);
			}
		});
	}

	function setPage(next: number) {
		updateParams((params) => params.set('page', String(next)), false);
	}

	function setPerPage(next: number) {
		updateParams((params) => params.set('per_page', String(next)));
	}
</script>

<div class="screen">
	<div class="head">
		<div class="row1">
			<h1><span class="kind">{typeLabel(data.type)}</span></h1>
			<span class="total">{formatNumber(data.entries.total)}</span>
		</div>
		<p class="sub">{t('list.subtitle', { type: typeLabel(data.type) })}</p>
	</div>

	<div class="toolbar">
		<div class="seg">
			<button class:on={view === 'table'} onclick={() => (view = 'table')}>
				{t('list.viewTable')}
			</button>
			<button class:on={view === 'list'} onclick={() => (view = 'list')}>
				{t('list.viewList')}
			</button>
		</div>
		<div class="tsep"></div>
		<SortMenu options={sortOptions} field={sort.field} descending={sort.descending} onchange={setSort} />
		<FilterMenu fields={filterFields} onadd={addFilter} />
	</div>

	<FilterChips {chips} onremove={removeFilter} onclearall={clearAllFilters} />

	{#if data.entries.items.length === 0}
		<div class="empty">
			{#if chips.length > 0}
				<p>{t('list.emptyFiltered')}</p>
				<Button variant="ghost" size="sm" onclick={clearAllFilters}>{t('list.clearFilters')}</Button>
			{:else}
				<p>{t('list.empty', { type: typeLabel(data.type) })}</p>
			{/if}
		</div>
	{:else if view === 'table'}
		<EntriesTable
			entries={data.entries.items}
			sortField={sort.field}
			sortDescending={sort.descending}
			onsort={toggleColumnSort}
		/>
	{:else}
		<EntriesCards entries={data.entries.items} />
	{/if}

	{#if data.entries.total > 0}
		<Pager
			page={data.entries.page}
			perPage={data.entries.per_page}
			total={data.entries.total}
			onpage={setPage}
			onperpage={setPerPage}
		/>
	{/if}
</div>

<style>
	.screen {
		padding: 20px 26px 40px;
	}
	.head h1 {
		margin: 0;
		font-size: 21px;
		letter-spacing: -0.02em;
		font-weight: 660;
	}
	.row1 {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.kind {
		color: var(--accent);
	}
	.total {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--muted);
		background: var(--surface-2);
		border: 1px solid var(--border);
		border-radius: 20px;
		padding: 1px 9px;
		font-variant-numeric: tabular-nums;
	}
	.sub {
		color: var(--muted);
		font-size: 12.5px;
		margin: 4px 0 0;
	}
	.toolbar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 14px 0 12px;
		flex-wrap: wrap;
	}
	.seg {
		display: inline-flex;
		background: var(--surface-2);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		padding: 2px;
		gap: 2px;
	}
	.seg button {
		height: 24px;
		padding: 0 10px;
		border: 0;
		background: transparent;
		color: var(--muted);
		border-radius: 4px;
		font-size: 12.5px;
		font: inherit;
		cursor: pointer;
	}
	.seg button.on {
		background: var(--surface);
		color: var(--text);
		box-shadow: var(--shadow);
		font-weight: 500;
	}
	.tsep {
		width: 1px;
		height: 22px;
		background: var(--border);
		margin: 0 2px;
	}
	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 60px 20px;
		text-align: center;
		color: var(--muted);
		font-size: 13.5px;
	}
	.empty p {
		margin: 0;
	}
</style>
