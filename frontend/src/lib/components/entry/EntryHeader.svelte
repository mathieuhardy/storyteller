<script lang="ts">
	// Entry header (docs/ui/screens.md §2): cover/portrait, type badge, title,
	// aliases, tags, and action buttons.
	import type { Entry } from '$api/types';
	import { t, typeLabel } from '$i18n/index.svelte';
	import Button from '$components/Button.svelte';
	import Icon from '$components/Icon.svelte';

	let { entry }: { entry: Entry } = $props();

	const title = $derived(
		typeof entry.frontmatter.title === 'string' ? entry.frontmatter.title : entry.slug
	);

	const aliases = $derived(
		Array.isArray(entry.frontmatter.aliases)
			? (entry.frontmatter.aliases as string[])
			: []
	);

	const tags = $derived(
		Array.isArray(entry.frontmatter.tags)
			? (entry.frontmatter.tags as string[])
			: []
	);

	const cover = $derived(
		typeof entry.frontmatter.cover === 'string' ? entry.frontmatter.cover : null
	);

	const portrait = $derived(
		typeof entry.frontmatter.portrait === 'string' ? entry.frontmatter.portrait : null
	);

	const image = $derived(portrait ?? cover);

	function initials(text: string): string {
		return text.slice(0, 2).toUpperCase();
	}
</script>

<header class="entry-header">
	<div class="top-row">
		<div class="identity">
			{#if image}
				<img class="avatar" src="/api/v1/assets/{image}" alt="" />
			{:else}
				<span class="avatar placeholder">{initials(title)}</span>
			{/if}

			<div class="info">
				<span class="type-badge">{typeLabel(entry.type)}</span>
				<h1 class="title">{title}</h1>
				{#if aliases.length > 0}
					<div class="aliases">
						{#each aliases as alias}
							<span class="alias">{alias}</span>
						{/each}
					</div>
				{/if}
			</div>
		</div>

		<div class="actions">
			<Button variant="primary" size="sm" href="/entry/{entry.slug}/edit">
				<Icon name="edit" size={14} />
				{t('entry.edit')}
			</Button>
			<Button variant="ghost" size="sm" onclick={() => {}}>
				<Icon name="external-link" size={14} />
				{t('entry.openFile')}
			</Button>
		</div>
	</div>

	{#if tags.length > 0}
		<div class="tags">
			{#each tags as tag}
				<a class="tag" href="/type/{entry.type}?tag={encodeURIComponent(tag)}">{tag}</a>
			{/each}
		</div>
	{/if}
</header>

<style>
	.entry-header {
		margin-bottom: 24px;
	}

	.top-row {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 20px;
	}

	.identity {
		display: flex;
		align-items: flex-start;
		gap: 16px;
		min-width: 0;
	}

	.avatar {
		flex-shrink: 0;
		width: 64px;
		height: 64px;
		border-radius: var(--radius);
		object-fit: cover;
	}

	.avatar.placeholder {
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-3);
		font-size: 18px;
		font-weight: 700;
		color: var(--muted);
	}

	.info {
		min-width: 0;
	}

	.type-badge {
		display: inline-block;
		padding: 2px 8px;
		background: var(--accent-soft);
		color: var(--accent);
		border-radius: 4px;
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		margin-bottom: 6px;
	}

	.title {
		margin: 0;
		font-size: 24px;
		font-weight: 700;
		letter-spacing: -0.02em;
		color: var(--text);
		line-height: 1.2;
	}

	.aliases {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		margin-top: 6px;
	}

	.alias {
		font-size: 12.5px;
		color: var(--muted);
		font-style: italic;
	}
	.alias::before {
		content: 'aka ';
		font-style: normal;
		color: var(--faint);
	}

	.actions {
		display: flex;
		gap: 8px;
		flex-shrink: 0;
	}

	.tags {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		margin-top: 14px;
		padding-top: 14px;
		border-top: 1px solid var(--border);
	}

	.tag {
		display: inline-block;
		padding: 3px 10px;
		background: var(--surface-2);
		border: 1px solid var(--border);
		border-radius: 12px;
		font-size: 12px;
		color: var(--muted);
		text-decoration: none;
		transition: background 0.1s ease;
	}
	.tag:hover {
		background: var(--surface-3);
		color: var(--text);
	}
</style>
