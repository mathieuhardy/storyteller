<script lang="ts">
	// Disambiguation modal (docs/ui/screens.md §5): pick the intended candidate,
	// preview the rewrite, and choose a scope — this one occurrence, or every
	// occurrence of the same ambiguous text. Rewrites the source entry/entries
	// non-destructively (docs/linking.md §3.3): only the matched wikilink text
	// changes, nothing else in the body/field is touched.
	import { invalidateAll } from '$app/navigation';
	import Modal from '$components/Modal.svelte';
	import Button from '$components/Button.svelte';
	import { t, typeLabel } from '$i18n/index.svelte';
	import { getEntity, updateEntity, ApiError } from '$api/client';
	import type { EntrySummary } from '$api/types';
	import { buildDisambiguationPatch, type AmbiguousGroup } from '$lib/workshop/link-rewrite';

	let {
		open = $bindable(false),
		group,
		candidates,
		onresolved
	}: {
		open?: boolean;
		group: AmbiguousGroup | null;
		candidates: EntrySummary[];
		onresolved: () => void;
	} = $props();

	let selectedSlug = $state<string | null>(null);
	let scope = $state<'this' | 'all'>('this');
	let processing = $state(false);
	let error = $state<string | null>(null);

	$effect(() => {
		if (open && group) {
			selectedSlug = group.candidateSlugs[0] ?? candidates[0]?.slug ?? null;
			scope = 'this';
			error = null;
		}
	});

	const selectedCandidate = $derived(candidates.find((c) => c.slug === selectedSlug) ?? null);

	function close() {
		open = false;
	}

	async function submit() {
		if (!group || !selectedSlug) return;
		const targeted =
			scope === 'this' ? group.occurrences.slice(0, 1) : group.occurrences;

		const bySource = new Map<string, (string | null)[]>();
		for (const occ of targeted) {
			const fields = bySource.get(occ.slug) ?? [];
			fields.push(occ.field);
			bySource.set(occ.slug, fields);
		}

		processing = true;
		error = null;
		try {
			for (const [sourceSlug, fields] of bySource) {
				const entry = await getEntity(sourceSlug);
				const patch = buildDisambiguationPatch(entry, fields, group.targetRaw, selectedSlug);
				if (patch.frontmatter || patch.body !== undefined) {
					await updateEntity(sourceSlug, patch);
				}
			}
			open = false;
			await invalidateAll();
			onresolved();
		} catch (e) {
			error = e instanceof ApiError ? e.message : String(e);
		} finally {
			processing = false;
		}
	}
</script>

{#if group}
	<Modal
		{open}
		kicker={t('workshop.ambiguousKicker')}
		title={`[[${group.targetRaw}]]`}
		tone="amb"
		onclose={close}
	>
		{#snippet body()}
			<p class="intro">{t('workshop.disambiguateIntro')}</p>

			{#each candidates as candidate (candidate.slug)}
				<button
					type="button"
					class="cand"
					class:sel={selectedSlug === candidate.slug}
					onclick={() => (selectedSlug = candidate.slug)}
				>
					<span class="radio"></span>
					<div class="ctext">
						<div class="cn">{candidate.title}</div>
					</div>
					<span class="ct">{typeLabel(candidate.type)}</span>
				</button>
			{/each}

			{#if selectedCandidate}
				<span class="flabel">{t('workshop.rewriteLabel')}</span>
				<div class="rewrite">
					<span class="old">[[{group.targetRaw}]]</span>
					→
					<span class="new">[[{selectedCandidate.slug}|{group.targetRaw}]]</span>
				</div>
			{/if}

			<div class="scope">
				<button type="button" class="scopeopt" class:sel={scope === 'this'} onclick={() => (scope = 'this')}>
					<div class="so-t">{t('workshop.scopeThis')}</div>
					<div class="so-s">{t('workshop.scopeThisHint')}</div>
				</button>
				<button type="button" class="scopeopt" class:sel={scope === 'all'} onclick={() => (scope = 'all')}>
					<div class="so-t">{t('workshop.scopeAll')}</div>
					<div class="so-s">{t('workshop.scopeAllHint', { count: group.occurrences.length })}</div>
				</button>
			</div>

			{#if error}<p class="error">{error}</p>{/if}
		{/snippet}
		{#snippet footer()}
			<span class="foothint">{t('workshop.nonDestructiveHint')}</span>
			<div class="sp"></div>
			<Button variant="ghost" size="sm" onclick={close}>{t('action.cancel')}</Button>
			<Button variant="primary" size="sm" onclick={submit} disabled={processing || !selectedSlug}>
				{processing ? t('editor.saving') : t('workshop.rewriteAction')}
			</Button>
		{/snippet}
	</Modal>
{/if}

<style>
	.intro {
		margin: 0 0 14px;
		color: var(--muted);
		font-size: 13px;
	}
	.cand {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px;
		border: 1px solid var(--border);
		border-radius: var(--radius);
		margin-bottom: 8px;
		cursor: pointer;
		background: var(--surface);
		font: inherit;
		text-align: left;
	}
	.cand:hover {
		border-color: var(--border-strong);
	}
	.cand.sel {
		border-color: var(--accent);
		background: var(--accent-soft);
	}
	.radio {
		width: 18px;
		height: 18px;
		border-radius: 50%;
		border: 2px solid var(--border-strong);
		flex: none;
		position: relative;
	}
	.cand.sel .radio {
		border-color: var(--accent);
	}
	.cand.sel .radio::after {
		content: '';
		position: absolute;
		inset: 3px;
		border-radius: 50%;
		background: var(--accent);
	}
	.ctext {
		flex: 1;
		min-width: 0;
	}
	.cn {
		font-weight: 600;
	}
	.ct {
		font-size: 10.5px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--faint);
	}
	.flabel {
		display: block;
		font-size: 10.5px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--faint);
		margin: 16px 0 6px;
	}
	.rewrite {
		font-family: var(--font-mono);
		font-size: 13px;
		background: var(--surface-2);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		padding: 11px 12px;
		color: var(--muted);
	}
	.rewrite .old {
		color: var(--warning);
		text-decoration: line-through;
		opacity: 0.7;
	}
	.rewrite .new {
		color: var(--accent);
	}
	.scope {
		display: flex;
		gap: 8px;
		margin-top: 14px;
	}
	.scopeopt {
		flex: 1;
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 10px 12px;
		cursor: pointer;
		font-size: 12.5px;
		background: var(--surface);
		font: inherit;
		text-align: left;
	}
	.scopeopt.sel {
		border-color: var(--accent);
		background: var(--accent-soft);
	}
	.so-t {
		font-weight: 600;
		color: var(--text);
	}
	.so-s {
		color: var(--muted);
		margin-top: 2px;
	}
	.error {
		color: var(--danger);
		font-size: 12.5px;
		margin: 12px 0 0;
	}
	.foothint {
		font-size: 12px;
		color: var(--muted);
	}
	.sp {
		flex: 1;
	}
</style>
