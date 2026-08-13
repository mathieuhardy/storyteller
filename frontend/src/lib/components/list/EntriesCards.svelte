<script lang="ts">
	// List view of the list/table screen (docs/ui/screens.md §3 /
	// components.md §5): cards with excerpt, tags and an error flag.
	import { t } from '$i18n/index.svelte';
	import type { EntrySummary } from '$api/types';

	let { entries }: { entries: EntrySummary[] } = $props();

	function initials(title: string): string {
		return title.slice(0, 2).toUpperCase();
	}
</script>

<div class="listwrap">
	{#each entries as entry (entry.slug)}
		<a class="lcard" href="/entry/{entry.slug}">
			<span class="av">{initials(entry.title)}</span>
			<div class="lc-body">
				<div class="lc-top">
					<span class="lc-title">{entry.title}</span>
					{#if entry.has_errors}
						<span class="warnflag">{t('list.hasErrors')}</span>
					{/if}
				</div>
				{#if entry.excerpt}<p class="lc-ex">{entry.excerpt}</p>{/if}
				{#if entry.tags.length > 0}
					<div class="lc-meta">
						{#each entry.tags as tag (tag)}
							<span class="tg">{tag}</span>
						{/each}
					</div>
				{/if}
			</div>
		</a>
	{/each}
</div>

<style>
	.listwrap {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.lcard {
		display: flex;
		gap: 14px;
		padding: 14px 16px;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		text-decoration: none;
		color: inherit;
	}
	.lcard:hover {
		border-color: var(--border-strong);
		background: var(--surface-2);
	}
	.av {
		width: 40px;
		height: 40px;
		border-radius: 8px;
		flex: none;
		display: grid;
		place-items: center;
		background: var(--surface-3);
		color: var(--muted);
		font-size: 15px;
		font-weight: 600;
	}
	.lc-body {
		min-width: 0;
		flex: 1;
	}
	.lc-top {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.lc-title {
		font-weight: 650;
		font-size: 15px;
	}
	.lc-ex {
		margin: 3px 0 0;
		color: var(--muted);
		font-size: 13px;
		display: -webkit-box;
		-webkit-line-clamp: 1;
		line-clamp: 1;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
	.lc-meta {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-top: 8px;
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
	.warnflag {
		color: var(--danger);
		display: inline-flex;
		align-items: center;
		font-size: 12px;
	}
</style>
