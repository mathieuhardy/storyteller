<script lang="ts">
	// Link / link-list field (docs/ui/components.md §3-4): existing targets as
	// clickable-to-remove tokens, plus an "add a link" popover offering
	// title/alias matches among entries of the field's `link_targets` types, and
	// a free-text "create …" option that just writes a new `[[Target]]` — a stub
	// until it is promoted to a real entry from the links workshop (no source
	// file is rewritten either way).
	import Popover from '$components/Popover.svelte';
	import { getEntities } from '$api/client';
	import { typeLabel, t } from '$i18n/index.svelte';
	import type { EntrySummary } from '$api/types';

	function focusOnMount(node: HTMLInputElement) {
		node.focus();
	}

	let {
		tokens = $bindable([]),
		linkTargets,
		multi,
		addLabel
	}: {
		tokens?: string[];
		linkTargets: string[];
		multi: boolean;
		addLabel: string;
	} = $props();

	let open = $state(false);
	let query = $state('');
	let candidates = $state<EntrySummary[]>([]);
	let loaded = false;

	async function ensureLoaded() {
		if (loaded) return;
		loaded = true;
		const params = new URLSearchParams();
		for (const type of linkTargets) params.append('type', type);
		params.set('per_page', '200');
		try {
			const page = await getEntities(`?${params.toString()}`);
			candidates = page.items;
		} catch {
			candidates = [];
		}
	}

	const matches = $derived.by(() => {
		const q = query.trim().toLowerCase();
		if (!q) return candidates.slice(0, 20);
		return candidates.filter((c) => c.title.toLowerCase().includes(q)).slice(0, 20);
	});

	const exactMatch = $derived(
		candidates.some((c) => c.title.toLowerCase() === query.trim().toLowerCase())
	);

	function add(title: string) {
		const value = title.trim();
		if (!value) return;
		tokens = multi ? [...tokens.filter((existing) => existing !== value), value] : [value];
		query = '';
		open = false;
	}

	function remove(index: number) {
		tokens = tokens.filter((_, i) => i !== index);
	}

	function onOpenToggle(toggle: () => void) {
		ensureLoaded();
		toggle();
	}
</script>

<div class="link-editor">
	{#each tokens as tokenValue, i (i)}
		<span class="token link">
			{tokenValue}
			<button type="button" class="remove" onclick={() => remove(i)} aria-label="remove">✕</button
			>
		</span>
	{/each}

	{#if multi || tokens.length === 0}
		<Popover bind:open align="left">
			{#snippet trigger({ toggle })}
				<button type="button" class="add-link" onclick={() => onOpenToggle(toggle)}>
					🔗 {addLabel}
				</button>
			{/snippet}
			<div class="ac-panel">
				<input
					class="ac-search"
					type="text"
					bind:value={query}
					placeholder={t('editor.linkSearchPlaceholder')}
					use:focusOnMount
				/>
				<div class="ac-list">
					{#each matches as candidate (candidate.slug)}
						<button type="button" class="ac-item" onclick={() => add(candidate.title)}>
							<span class="ac-title">{candidate.title}</span>
							<span class="ac-type">{typeLabel(candidate.type)}</span>
						</button>
					{/each}
					{#if query.trim() && !exactMatch}
						<button type="button" class="ac-item create" onclick={() => add(query)}>
							<span class="plus">＋</span>
							{t('editor.createStubOption', { title: query.trim() })}
						</button>
					{/if}
				</div>
			</div>
		</Popover>
	{/if}
</div>

<style>
	.link-editor {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		align-items: center;
	}
	.token {
		height: 22px;
		padding: 0 4px 0 9px;
		display: inline-flex;
		align-items: center;
		gap: 6px;
		border-radius: 6px;
		font-size: 12px;
		border: 1px solid var(--border);
		color: var(--text);
	}
	.token.link {
		background: var(--accent-soft);
		border-color: var(--accent-line);
		color: var(--accent);
	}
	.remove {
		border: 0;
		background: transparent;
		color: inherit;
		opacity: 0.7;
		cursor: pointer;
		font-size: 11px;
		padding: 0;
		line-height: 1;
	}
	.remove:hover {
		opacity: 1;
	}
	.add-link {
		height: 30px;
		padding: 0 10px;
		display: inline-flex;
		align-items: center;
		gap: 6px;
		cursor: pointer;
		border: 1px dashed var(--border-strong);
		border-radius: var(--radius-sm);
		color: var(--muted);
		font-size: 12.5px;
		background: transparent;
		font: inherit;
	}
	.add-link:hover {
		border-color: var(--accent);
		color: var(--accent);
	}
	.ac-panel {
		width: 260px;
	}
	.ac-search {
		width: 100%;
		height: 30px;
		padding: 0 9px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--surface-2);
		color: var(--text);
		font: inherit;
		font-size: 13px;
		margin-bottom: 4px;
	}
	.ac-search:focus {
		outline: none;
		border-color: var(--accent-line);
	}
	.ac-list {
		max-height: 240px;
		overflow-y: auto;
	}
	.ac-item {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 9px;
		padding: 7px 9px;
		border: 0;
		background: transparent;
		border-radius: 5px;
		font-size: 13px;
		font: inherit;
		color: var(--text);
		text-align: left;
		cursor: pointer;
	}
	.ac-item:hover {
		background: var(--surface-2);
	}
	.ac-title {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.ac-type {
		font-size: 10px;
		color: var(--faint);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}
	.ac-item.create {
		color: var(--stub);
	}
	.plus {
		width: 18px;
		text-align: center;
	}
</style>
