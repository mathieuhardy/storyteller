<script lang="ts">
	// Project hero section (docs/ui/screens.md §1). Shows project.md entry with
	// cover, title, logline, status pill and genre chips. Gracefully handles
	// missing entry or missing fields.
	import type { Entry } from '$api/types';
	import { t, typeLabel } from '$i18n/index.svelte';

	let {
		entry,
		projectRoot
	}: {
		entry: Entry | null;
		projectRoot: string;
	} = $props();

	// Extract fields safely from frontmatter
	const fm = $derived(entry?.frontmatter ?? {});
	const title = $derived((fm.title as string) ?? 'Untitled Project');
	const logline = $derived((fm.logline as string) ?? '');
	const status = $derived((fm.status as string) ?? '');
	const genres = $derived(
		Array.isArray(fm.genres) ? (fm.genres as string[]) : []
	);
	const cover = $derived((fm.cover as string) ?? '');

	// Project name from root path as fallback
	const projectName = $derived(projectRoot.split('/').pop() ?? 'Project');
</script>

<section class="hero">
	<div class="cover" class:has-image={!!cover}>
		{#if cover}
			<img src="/api/v1/assets/{cover}" alt="" />
		{:else}
			<span class="cover-placeholder"></span>
		{/if}
	</div>

	<div class="info">
		<span class="type-badge">
			<span class="type-icon">P</span>
			{typeLabel('project')}
		</span>

		<h1 class="title">{title}</h1>

		{#if logline}
			<p class="logline">{logline}</p>
		{/if}

		<div class="meta">
			{#if status}
				<span class="status-pill">
					<span class="status-dot"></span>
					{status}
				</span>
			{/if}

			{#each genres as genre}
				<span class="genre-chip">{genre}</span>
			{/each}

			{#if entry?.path}
				<span class="file-ref">{entry.path}</span>
			{/if}
		</div>
	</div>
</section>

<style>
	.hero {
		display: flex;
		gap: 22px;
		margin-bottom: 24px;
	}

	.cover {
		flex-shrink: 0;
		width: 92px;
		height: 118px;
		border-radius: var(--radius-lg);
		background: linear-gradient(135deg, var(--surface-2), var(--surface-3));
		box-shadow: var(--shadow-lg);
		overflow: hidden;
	}
	.cover img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.cover-placeholder {
		display: block;
		width: 100%;
		height: 100%;
		background: linear-gradient(
			135deg,
			var(--accent-soft) 0%,
			var(--surface-3) 100%
		);
	}

	.info {
		flex: 1;
		min-width: 0;
	}

	.type-badge {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 10px;
		background: var(--accent-soft);
		color: var(--accent);
		border-radius: 20px;
		font-size: 12px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}
	.type-icon {
		width: 16px;
		height: 16px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--accent);
		color: var(--accent-fg);
		border-radius: 4px;
		font-size: 10px;
		font-weight: 700;
	}

	.title {
		margin: 10px 0 6px;
		font-size: 28px;
		font-weight: 700;
		color: var(--text);
		line-height: 1.2;
	}

	.logline {
		margin: 0 0 12px;
		font-size: 15.5px;
		font-weight: 400;
		font-style: italic;
		color: var(--text);
		max-width: 56ch;
		line-height: 1.5;
	}

	.meta {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}

	.status-pill {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		height: 24px;
		padding: 0 11px;
		background: var(--ok-soft);
		color: var(--ok);
		border-radius: 20px;
		font-size: 12.5px;
		font-weight: 600;
	}
	.status-dot {
		width: 6px;
		height: 6px;
		background: var(--ok);
		border-radius: 50%;
	}

	.genre-chip {
		display: inline-flex;
		align-items: center;
		height: 22px;
		padding: 0 9px;
		background: var(--surface-2);
		border: 1px solid var(--border);
		border-radius: 6px;
		font-size: 12px;
		color: var(--muted);
	}

	.file-ref {
		font-family: var(--font-mono);
		font-size: 11.5px;
		color: var(--faint);
	}

	/* Responsive: stack on narrow screens */
	@media (max-width: 820px) {
		.hero {
			flex-direction: column;
			align-items: flex-start;
		}
	}
</style>
