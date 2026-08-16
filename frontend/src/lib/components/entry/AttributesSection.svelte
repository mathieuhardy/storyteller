<script lang="ts">
	// Attributes section (docs/ui/screens.md §2): displays frontmatter fields
	// rendered by kind, with labels from the type schema. Unknown keys are shown
	// in a separate "preserved fields" block.
	import type { FieldSchema, OutgoingLink, Frontmatter } from '$api/types';
	import { t, fieldLabel } from '$i18n/index.svelte';
	import LinkToken from '$components/entry/LinkToken.svelte';

	let {
		frontmatter,
		fields,
		links
	}: {
		frontmatter: Frontmatter;
		fields: FieldSchema[];
		links: OutgoingLink[];
	} = $props();

	// Fields to never display (handled in header or implicit)
	const HIDDEN_FIELDS = new Set(['type', 'title', 'aliases', 'tags', 'cover', 'portrait']);

	const knownFieldNames = $derived(new Set(fields.map((f) => f.name)));

	const visibleFields = $derived(
		fields.filter((f) => !HIDDEN_FIELDS.has(f.name) && frontmatter[f.name] !== undefined)
	);

	const unknownFields = $derived(
		Object.entries(frontmatter).filter(
			([key]) => !knownFieldNames.has(key) && !HIDDEN_FIELDS.has(key)
		)
	);

	// Build a map of link resolutions for quick lookup
	const linkResolutions = $derived(
		new Map(links.map((l) => [l.target_raw, l]))
	);

	function isLink(kind: string): boolean {
		return kind === 'link' || kind === 'link-list';
	}

	function extractLinkTarget(value: unknown): string | null {
		if (typeof value !== 'string') return null;
		const match = value.match(/^\[\[(.+?)\]\]$/);
		return match ? match[1] : null;
	}

	function formatValue(value: unknown, kind: string): string {
		if (value === null || value === undefined) return '';
		if (typeof value === 'boolean') return value ? t('entry.yes') : t('entry.no');
		if (Array.isArray(value)) return value.map((v) => String(v)).join(', ');
		return String(value);
	}
</script>

{#if visibleFields.length > 0 || unknownFields.length > 0}
	<section class="attributes">
		<h2 class="section-title">{t('entry.attributes')}</h2>

		<dl class="fields">
			{#each visibleFields as field}
				{@const value = frontmatter[field.name]}
				{@const linkTarget = isLink(field.kind) ? extractLinkTarget(value) : null}
				<div class="field">
					<dt class="label">{fieldLabel(field)}</dt>
					<dd class="value">
						{#if field.kind === 'link' && linkTarget}
							{@const resolution = linkResolutions.get(linkTarget)}
							<LinkToken target={linkTarget} {resolution} />
						{:else if field.kind === 'link-list' && Array.isArray(value)}
							<div class="link-list">
								{#each value as item}
									{@const target = extractLinkTarget(item)}
									{#if target}
										{@const resolution = linkResolutions.get(target)}
										<LinkToken {target} {resolution} />
									{:else}
										<span class="raw-value">{String(item)}</span>
									{/if}
								{/each}
							</div>
						{:else if field.kind === 'boolean'}
							<span class="bool-value" class:is-true={value === true}>
								{value ? t('entry.yes') : t('entry.no')}
							</span>
						{:else if field.kind === 'list' && Array.isArray(value)}
							<div class="list-value">
								{#each value as item}
									<span class="list-item">{String(item)}</span>
								{/each}
							</div>
						{:else}
							<span class="text-value">{formatValue(value, field.kind)}</span>
						{/if}
					</dd>
				</div>
			{/each}
		</dl>

		{#if unknownFields.length > 0}
			<details class="preserved">
				<summary class="preserved-header">
					<span class="lock">🔒</span>
					{t('entry.preservedFields')}
					<span class="count">{unknownFields.length}</span>
				</summary>
				<dl class="fields preserved-fields">
					{#each unknownFields as [key, value]}
						<div class="field">
							<dt class="label mono">{key}</dt>
							<dd class="value mono">{JSON.stringify(value)}</dd>
						</div>
					{/each}
				</dl>
			</details>
		{/if}
	</section>
{/if}

<style>
	.attributes {
		margin-bottom: 28px;
	}

	.section-title {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--muted);
		margin: 0 0 14px;
		padding-bottom: 8px;
		border-bottom: 1px solid var(--border);
	}

	.fields {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
		gap: 14px 24px;
		margin: 0;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.label {
		font-size: 11px;
		font-weight: 500;
		color: var(--faint);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.value {
		margin: 0;
		font-size: 14px;
		color: var(--text);
	}

	.text-value {
		line-height: 1.5;
	}

	.bool-value {
		padding: 2px 8px;
		background: var(--surface-2);
		border-radius: 4px;
		font-size: 12px;
	}
	.bool-value.is-true {
		background: var(--accent-soft);
		color: var(--accent);
	}

	.link-list {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.list-value {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.list-item {
		padding: 2px 8px;
		background: var(--surface-2);
		border: 1px solid var(--border);
		border-radius: 4px;
		font-size: 12.5px;
	}

	.preserved {
		margin-top: 20px;
		padding: 12px 14px;
		background: var(--surface-2);
		border: 1px solid var(--border);
		border-radius: var(--radius);
	}

	.preserved-header {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 12px;
		color: var(--muted);
		cursor: pointer;
	}

	.preserved-header:hover {
		color: var(--text);
	}

	.lock {
		font-size: 11px;
	}

	.count {
		font-family: var(--font-mono);
		font-size: 10px;
		background: var(--surface-3);
		padding: 1px 6px;
		border-radius: 10px;
	}

	.preserved-fields {
		margin-top: 12px;
	}

	.mono {
		font-family: var(--font-mono);
		font-size: 12px;
	}
</style>
