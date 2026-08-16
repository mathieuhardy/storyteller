<script lang="ts">
	// Collapsible right panel (docs/ui/layout.md §2.4). When viewing an entry, it
	// shows backlinks (mentioned in), outgoing links, and metadata. Otherwise it
	// shows a placeholder message.
	import Icon from './Icon.svelte';
	import IconButton from './IconButton.svelte';
	import { t, typeLabel, formatDate, fieldLabel } from '$i18n/index.svelte';
	import { getRailState } from '$stores/rail.svelte';
	import type { Backlink, OutgoingLink } from '$api/types';

	let { onClose }: { onClose?: () => void } = $props();

	const rail = $derived(getRailState());
	const hasEntry = $derived(rail.entry !== null);

	// Group backlinks by source entry
	const groupedBacklinks = $derived.by(() => {
		if (!rail.backlinks.length) return [];
		const groups = new Map<string, Backlink[]>();
		for (const bl of rail.backlinks) {
			const existing = groups.get(bl.slug) ?? [];
			existing.push(bl);
			groups.set(bl.slug, existing);
		}
		return Array.from(groups.entries()).map(([slug, items]) => ({
			slug,
			title: items[0].title,
			type: items[0].type,
			items
		}));
	});

	function linkStateClass(resolution: string): string {
		if (resolution === 'resolved') return 'resolved';
		if (resolution === 'stub') return 'stub';
		return 'ambiguous';
	}
</script>

<aside class="rail" aria-label={t('links.toggle')}>
	<div class="head">
		<span class="title">{t('links.toggle')}</span>
		<IconButton label={t('action.cancel')} onclick={() => onClose?.()}>
			<Icon name="x" size={15} />
		</IconButton>
	</div>

	{#if hasEntry}
		<div class="body">
			<!-- Backlinks: Mentioned in -->
			<section class="section">
				<h3 class="section-title">
					{t('entry.backlinks')}
					{#if rail.backlinks.length > 0}
						<span class="count">{rail.backlinks.length}</span>
					{/if}
				</h3>
				{#if groupedBacklinks.length === 0}
					<p class="empty">{t('entry.noBacklinks')}</p>
				{:else}
					<ul class="backlink-list">
						{#each groupedBacklinks as group}
							<li class="backlink-group">
								<a class="backlink-source" href="/entry/{group.slug}">
									<span class="source-type">{typeLabel(group.type)}</span>
									<span class="source-title">{group.title}</span>
								</a>
								<ul class="backlink-refs">
									{#each group.items as item}
										<li class="backlink-ref">
											<span class="ref-field">
												{item.field ? t('entry.viaField', { field: item.field }) : t('entry.inBody')}
											</span>
											{#if item.context}
												<span class="ref-context">{item.context}</span>
											{/if}
										</li>
									{/each}
								</ul>
							</li>
						{/each}
					</ul>
				{/if}
			</section>

			<!-- Outgoing links -->
			<section class="section">
				<h3 class="section-title">
					{t('entry.outgoingLinks')}
					{#if rail.links.length > 0}
						<span class="count">{rail.links.length}</span>
					{/if}
				</h3>
				{#if rail.links.length === 0}
					<p class="empty">{t('entry.noOutgoingLinks')}</p>
				{:else}
					<ul class="link-list">
						{#each rail.links as link}
							<li class="link-item">
								<span class="link-dot {linkStateClass(link.resolution)}"></span>
								{#if link.resolution === 'resolved' && link.target_slug}
									<a class="link-target resolved" href="/entry/{link.target_slug}">
										{link.target_raw}
									</a>
								{:else}
									<span class="link-target {linkStateClass(link.resolution)}">
										{link.target_raw}
										{#if link.resolution === 'stub'}
											<sup>+</sup>
										{:else if link.resolution === 'ambiguous'}
											<sup>?</sup>
										{/if}
									</span>
								{/if}
								{#if link.field}
									<span class="link-field">{link.field}</span>
								{/if}
							</li>
						{/each}
					</ul>
				{/if}
			</section>

			<!-- Metadata -->
			<section class="section">
				<h3 class="section-title">{t('entry.metadata')}</h3>
				<dl class="meta-list">
					<div class="meta-item">
						<dt>{t('entry.path')}</dt>
						<dd class="mono">{rail.entry?.path}</dd>
					</div>
					{#if rail.entry?.frontmatter.created}
						<div class="meta-item">
							<dt>{t('entry.created')}</dt>
							<dd>{formatDate(String(rail.entry.frontmatter.created))}</dd>
						</div>
					{/if}
					{#if rail.entry?.frontmatter.updated}
						<div class="meta-item">
							<dt>{t('entry.updated')}</dt>
							<dd>{formatDate(String(rail.entry.frontmatter.updated))}</dd>
						</div>
					{/if}
				</dl>
			</section>
		</div>
	{:else}
		<div class="body">
			<p class="hint">{t('screen.comingSoonBody')}</p>
		</div>
	{/if}
</aside>

<style>
	.rail {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: var(--surface);
		border-left: 1px solid var(--border);
		overflow: hidden;
	}
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: 44px;
		padding: 0 10px 0 14px;
		border-bottom: 1px solid var(--border);
		flex-shrink: 0;
	}
	.title {
		font-weight: 600;
		font-size: 13px;
	}
	.body {
		padding: 14px;
		overflow-y: auto;
		flex: 1;
	}
	.hint {
		margin: 0;
		color: var(--faint);
		font-size: 12.5px;
		line-height: 1.6;
	}

	.section {
		margin-bottom: 20px;
	}
	.section:last-child {
		margin-bottom: 0;
	}

	.section-title {
		display: flex;
		align-items: center;
		gap: 8px;
		margin: 0 0 10px;
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--muted);
	}

	.count {
		font-family: var(--font-mono);
		font-size: 9px;
		background: var(--surface-2);
		padding: 1px 6px;
		border-radius: 10px;
	}

	.empty {
		margin: 0;
		font-size: 12px;
		color: var(--faint);
		font-style: italic;
	}

	/* Backlinks */
	.backlink-list {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.backlink-group {
		margin-bottom: 12px;
	}
	.backlink-group:last-child {
		margin-bottom: 0;
	}

	.backlink-source {
		display: flex;
		flex-direction: column;
		text-decoration: none;
		padding: 6px 8px;
		background: var(--surface-2);
		border-radius: var(--radius-sm);
		transition: background 0.1s ease;
	}
	.backlink-source:hover {
		background: var(--surface-3);
	}

	.source-type {
		font-size: 9px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--faint);
	}
	.source-title {
		font-size: 13px;
		font-weight: 500;
		color: var(--text);
	}

	.backlink-refs {
		list-style: none;
		margin: 6px 0 0 12px;
		padding: 0;
	}

	.backlink-ref {
		font-size: 11px;
		color: var(--muted);
		padding: 2px 0;
	}

	.ref-field {
		font-style: italic;
	}

	.ref-context {
		display: block;
		margin-top: 2px;
		font-size: 11px;
		color: var(--faint);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	/* Outgoing links */
	.link-list {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.link-item {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 0;
		font-size: 12.5px;
	}

	.link-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		flex-shrink: 0;
	}
	.link-dot.resolved {
		background: var(--accent);
	}
	.link-dot.stub {
		background: var(--stub);
	}
	.link-dot.ambiguous {
		background: var(--warning);
	}

	.link-target {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.link-target.resolved {
		color: var(--accent);
		text-decoration: none;
	}
	.link-target.resolved:hover {
		text-decoration: underline;
	}
	.link-target.stub {
		color: var(--stub);
	}
	.link-target.ambiguous {
		color: var(--warning);
	}
	.link-target sup {
		font-size: 9px;
		font-weight: 700;
	}

	.link-field {
		font-size: 10px;
		color: var(--faint);
		font-family: var(--font-mono);
	}

	/* Metadata */
	.meta-list {
		margin: 0;
	}

	.meta-item {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 6px 0;
		border-bottom: 1px solid var(--border);
	}
	.meta-item:last-child {
		border-bottom: none;
	}

	.meta-item dt {
		font-size: 10px;
		font-weight: 500;
		color: var(--faint);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.meta-item dd {
		margin: 0;
		font-size: 12px;
		color: var(--text);
	}

	.mono {
		font-family: var(--font-mono);
		font-size: 11px;
		word-break: break-all;
	}
</style>
