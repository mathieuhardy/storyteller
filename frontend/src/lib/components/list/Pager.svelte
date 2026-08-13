<script lang="ts">
	// Pagination bar (docs/api.md §4): `page`/`per_page`, capped server-side at
	// 200. Shows a small window of page numbers around the current one so it
	// stays usable with hundreds of pages.
	import Icon from '$components/Icon.svelte';
	import { t, formatNumber } from '$i18n/index.svelte';

	let {
		page,
		perPage,
		total,
		onpage,
		onperpage
	}: {
		page: number;
		perPage: number;
		total: number;
		onpage: (page: number) => void;
		onperpage: (perPage: number) => void;
	} = $props();

	const PER_PAGE_OPTIONS = [25, 50, 100, 200];

	const pageCount = $derived(Math.max(1, Math.ceil(total / perPage)));

	const windowPages = $derived.by(() => {
		const span = 2;
		const start = Math.max(1, page - span);
		const end = Math.min(pageCount, page + span);
		const pages: number[] = [];
		for (let p = start; p <= end; p++) pages.push(p);
		return pages;
	});
</script>

<div class="pager">
	<span>{t('list.totalCount', { count: formatNumber(total) })}</span>
	<div class="pspacer"></div>
	{#if pageCount > 1}
		<button
			class="pgbtn"
			disabled={page <= 1}
			onclick={() => onpage(page - 1)}
			aria-label={t('list.pagePrev')}
		>
			<Icon name="chevron-left" size={13} />
		</button>
		{#if windowPages[0] > 1}
			<button class="pgbtn" onclick={() => onpage(1)}>1</button>
			{#if windowPages[0] > 2}<span class="ellipsis">…</span>{/if}
		{/if}
		{#each windowPages as p (p)}
			<button class="pgbtn" class:on={p === page} onclick={() => onpage(p)}>{p}</button>
		{/each}
		{#if windowPages[windowPages.length - 1] < pageCount}
			{#if windowPages[windowPages.length - 1] < pageCount - 1}<span class="ellipsis">…</span>{/if}
			<button class="pgbtn" onclick={() => onpage(pageCount)}>{pageCount}</button>
		{/if}
		<button
			class="pgbtn"
			disabled={page >= pageCount}
			onclick={() => onpage(page + 1)}
			aria-label={t('list.pageNext')}
		>
			<Icon name="chevron-right" size={13} />
		</button>
	{/if}
	<span class="sep">·</span>
	<label class="perpage-label">
		<span class="k">{t('list.perPage')}:</span>
		<select
			class="perpage"
			value={perPage}
			onchange={(e) => onperpage(Number((e.currentTarget as HTMLSelectElement).value))}
		>
			{#each PER_PAGE_OPTIONS as option (option)}
				<option value={option}>{option}</option>
			{/each}
		</select>
	</label>
</div>

<style>
	.pager {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 16px 0 8px;
		color: var(--muted);
		font-size: 12.5px;
	}
	.pspacer {
		flex: 1;
	}
	.pgbtn {
		height: 28px;
		min-width: 28px;
		padding: 0 8px;
		border: 1px solid var(--border);
		background: var(--surface);
		border-radius: var(--radius-sm);
		color: var(--text);
		cursor: pointer;
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}
	.pgbtn.on {
		background: var(--accent-soft);
		border-color: var(--accent-line);
		color: var(--accent);
		font-weight: 600;
	}
	.pgbtn:disabled {
		color: var(--faint);
		cursor: default;
	}
	.ellipsis {
		color: var(--faint);
		padding: 0 2px;
	}
	.sep {
		color: var(--faint);
	}
	.perpage-label {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}
	.k {
		color: var(--faint);
	}
	.perpage {
		height: 28px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--text);
		font: inherit;
		font-family: var(--font-mono);
		font-size: 12.5px;
		padding: 0 6px;
		cursor: pointer;
	}
</style>
