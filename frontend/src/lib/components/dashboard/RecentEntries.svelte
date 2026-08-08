<script lang="ts">
	// Recent entries card (docs/ui/screens.md §1). Shows up to 5 recently modified
	// entries with avatar, title, type label, and error indicator. Links to full
	// entry view (placeholder route for now).
	import type { EntrySummary } from '$api/types';
	import { t, typeLabel } from '$i18n/index.svelte';

	let {
		entries
	}: {
		entries: EntrySummary[];
	} = $props();

	// Generate initials from title (first 2 letters, uppercase)
	function initials(title: string): string {
		return title.slice(0, 2).toUpperCase();
	}
</script>

<div class="card">
	<header class="header">
		<h2 class="header-title">{t('dashboard.recentlyModified')}</h2>
		<a class="view-all" href="/type/note">{t('dashboard.viewAll')} →</a>
	</header>

	{#if entries.length === 0}
		<div class="empty">{t('dashboard.noEntries')}</div>
	{:else}
		<ul class="list">
			{#each entries as entry}
				<li class="entry">
					<a class="entry-link" href="/entry/{entry.slug}">
						<span class="avatar">{initials(entry.title)}</span>
						<div class="entry-info">
							<span class="entry-title">
								{entry.title}
								{#if entry.has_errors}
									<span class="error-flag" title="Has errors">▲</span>
								{/if}
							</span>
							<span class="entry-type">{typeLabel(entry.type)}</span>
						</div>
					</a>
				</li>
			{/each}
		</ul>
	{/if}
</div>

<style>
	.card {
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		overflow: hidden;
	}

	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 15px;
		border-bottom: 1px solid var(--border);
	}
	.header-title {
		margin: 0;
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--muted);
	}
	.view-all {
		font-size: 12px;
		color: var(--accent);
		text-decoration: none;
	}
	.view-all:hover {
		text-decoration: underline;
	}

	.empty {
		padding: 24px 15px;
		text-align: center;
		font-size: 13px;
		color: var(--muted);
	}

	.list {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.entry {
		border-bottom: 1px solid var(--border);
	}
	.entry:last-child {
		border-bottom: none;
	}

	.entry-link {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 15px;
		text-decoration: none;
		color: inherit;
		transition: background 0.1s ease;
	}
	.entry-link:hover {
		background: var(--surface-2);
	}

	.avatar {
		flex-shrink: 0;
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-3);
		border-radius: 6px;
		font-size: 11px;
		font-weight: 600;
		color: var(--muted);
	}

	.entry-info {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.entry-title {
		font-size: 13.5px;
		font-weight: 600;
		color: var(--text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.error-flag {
		color: var(--danger);
		font-size: 10px;
		margin-left: 4px;
	}

	.entry-type {
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--faint);
	}
</style>
