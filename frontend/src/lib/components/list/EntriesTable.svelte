<script lang="ts">
	// Table view of the list/table screen (docs/ui/screens.md §3 /
	// components.md §5). Columns are limited to what `EntrySummary` actually
	// carries (state, name, tags, modified) — per-type frontmatter columns
	// (role, status…) need a richer list endpoint and are deferred.
	import { goto } from '$app/navigation';
	import { t, formatDate } from '$i18n/index.svelte';
	import type { EntrySummary } from '$api/types';

	let {
		entries,
		sortField,
		sortDescending,
		onsort
	}: {
		entries: EntrySummary[];
		sortField: string;
		sortDescending: boolean;
		onsort: (field: string) => void;
	} = $props();

	function initials(title: string): string {
		return title.slice(0, 2).toUpperCase();
	}

	function arrow(field: string): string {
		if (sortField !== field) return '';
		return sortDescending ? '▾' : '▴';
	}

	function open(slug: string) {
		goto(`/entry/${slug}`);
	}

	function onRowKeydown(event: KeyboardEvent, slug: string) {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			open(slug);
		}
	}
</script>

<div class="tablewrap">
	<div class="tscroll">
		<table>
			<thead>
				<tr>
					<th class="state"></th>
					<th onclick={() => onsort('title')}>{t('list.columnName')} <span class="ar">{arrow('title')}</span></th>
					<th>{t('list.columnTags')}</th>
					<th onclick={() => onsort('updated')}>{t('list.columnModified')} <span class="ar">{arrow('updated')}</span></th>
				</tr>
			</thead>
			<tbody>
				{#each entries as entry (entry.slug)}
					<tr
						tabindex="0"
						role="link"
						onclick={() => open(entry.slug)}
						onkeydown={(e) => onRowKeydown(e, entry.slug)}
					>
						<td class="state">
							{#if entry.has_errors}
								<span class="err" title={t('list.hasErrors')}>▲</span>
							{:else}
								<span class="ok" title={t('list.noErrors')}>●</span>
							{/if}
						</td>
						<td>
							<div class="cell-name">
								<span class="av">{initials(entry.title)}</span>
								<a href="/entry/{entry.slug}" onclick={(e) => e.stopPropagation()}>
									{entry.title}
								</a>
							</div>
						</td>
						<td>
							<div class="tagcell">
								{#each entry.tags as tag (tag)}
									<span class="tg">{tag}</span>
								{/each}
							</div>
						</td>
						<td class="mono">{entry.updated ? formatDate(entry.updated) : '—'}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
</div>

<style>
	.tablewrap {
		border: 1px solid var(--border);
		border-radius: var(--radius);
		overflow: hidden;
		background: var(--surface);
	}
	.tscroll {
		overflow-x: auto;
	}
	table {
		border-collapse: collapse;
		width: 100%;
		font-size: 13px;
	}
	thead th {
		text-align: left;
		font-weight: 600;
		color: var(--muted);
		font-size: 11px;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		padding: 9px 12px;
		background: var(--surface-2);
		border-bottom: 1px solid var(--border);
		white-space: nowrap;
		cursor: pointer;
		user-select: none;
	}
	thead th.state {
		width: 34px;
		padding-left: 14px;
		cursor: default;
	}
	.ar {
		color: var(--accent);
		margin-left: 3px;
	}
	tbody td {
		padding: 10px 12px;
		border-bottom: 1px solid var(--border);
		vertical-align: middle;
	}
	tbody tr:last-child td {
		border-bottom: 0;
	}
	tbody tr {
		cursor: pointer;
	}
	tbody tr:hover td {
		background: var(--surface-2);
	}
	tbody tr:focus-visible td {
		background: var(--accent-soft);
	}
	td.state {
		padding-left: 14px;
	}
	.ok {
		color: var(--faint);
	}
	.err {
		color: var(--danger);
	}
	.cell-name {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.cell-name a {
		font-weight: 600;
		color: var(--text);
		text-decoration: none;
	}
	.cell-name a:hover {
		color: var(--accent);
	}
	.av {
		width: 26px;
		height: 26px;
		border-radius: 6px;
		flex: none;
		display: grid;
		place-items: center;
		background: var(--surface-3);
		color: var(--muted);
		font-size: 12px;
		font-weight: 600;
	}
	.mono {
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
		color: var(--muted);
		font-size: 12px;
	}
	.tagcell {
		display: flex;
		gap: 4px;
		flex-wrap: wrap;
	}
	.tg {
		height: 19px;
		padding: 0 7px;
		border-radius: 5px;
		font-size: 11px;
		background: var(--surface-2);
		border: 1px solid var(--border);
		color: var(--text);
		display: inline-flex;
		align-items: center;
	}
</style>
