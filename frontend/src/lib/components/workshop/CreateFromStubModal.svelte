<script lang="ts">
	// Create-from-stub modal (docs/ui/screens.md §5): title pre-filled from the
	// stub's first label, a type picker grid, and a preview of the links that
	// resolve at the next reindex — plain `POST /entities`, no source file is
	// rewritten (docs/linking.md §6.3).
	import { goto } from '$app/navigation';
	import Modal from '$components/Modal.svelte';
	import Button from '$components/Button.svelte';
	import { t, typeLabel } from '$i18n/index.svelte';
	import { createEntity, ApiError } from '$api/client';
	import type { Stub, TypeResponse } from '$api/types';

	let {
		open = $bindable(false),
		stub,
		types,
		titlesBySlug
	}: {
		open?: boolean;
		stub: Stub | null;
		types: TypeResponse[];
		titlesBySlug: Record<string, string>;
	} = $props();

	let title = $state('');
	let selectedType = $state<string | null>(null);
	let creating = $state(false);
	let error = $state<string | null>(null);

	$effect(() => {
		if (open && stub) {
			title = stub.labels[0] ?? stub.key;
			selectedType = null;
			error = null;
		}
	});

	function slugPreview(text: string): string {
		const folded = text
			.normalize('NFD')
			.replace(new RegExp('[' + String.fromCharCode(0x0300) + '-' + String.fromCharCode(0x036f) + ']', 'g'), '')
			.toLowerCase();
		let out = '';
		let pendingDash = false;
		for (const ch of folded) {
			if (/[a-z0-9]/.test(ch)) {
				if (pendingDash && out) out += '-';
				pendingDash = false;
				out += ch;
			} else {
				pendingDash = true;
			}
		}
		return out;
	}

	const slug = $derived(slugPreview(title));
	const folder = $derived(types.find((tp) => tp.name === selectedType)?.folder ?? '…');

	function close() {
		open = false;
	}

	async function submit() {
		if (!stub) return;
		if (!title.trim() || !selectedType) {
			error = t('workshop.errorPickType');
			return;
		}
		creating = true;
		error = null;
		try {
			const created = await createEntity({ type: selectedType, title: title.trim() });
			open = false;
			await goto(`/entry/${created.slug}/edit`);
		} catch (e) {
			error = e instanceof ApiError ? e.message : String(e);
		} finally {
			creating = false;
		}
	}
</script>

{#if stub}
	<Modal
		{open}
		kicker={t('workshop.createKicker')}
		title={t('workshop.createTitle')}
		onclose={close}
	>
		{#snippet body()}
			<label class="flabel" for="stub-create-title">{t('workshop.titleLabel')}</label>
			<input id="stub-create-title" class="tin" type="text" bind:value={title} />
			<p class="slugprev">
				{t('workshop.identityPreview')} <b>{folder}/{slug || '…'}.md</b>
			</p>

			<span class="flabel typegrid-label" id="stub-create-type-label">{t('workshop.typeLabel')}</span>
			<div class="typegrid" role="group" aria-labelledby="stub-create-type-label">
				{#each types as tp (tp.name)}
					<button
						type="button"
						class="tcard"
						class:sel={selectedType === tp.name}
						onclick={() => (selectedType = tp.name)}
					>
						<div class="tn">{typeLabel(tp.name)}</div>
						<div class="tf">{tp.folder}/</div>
					</button>
				{/each}
			</div>

			{#if stub.count > 0}
				<div class="resolvebox">
					<div class="rh">
						<b>{t('workshop.linksWillResolve', { count: stub.count })}</b>
					</div>
					{#each stub.sources as sourceSlug (sourceSlug)}
						<div class="rl">{titlesBySlug[sourceSlug] ?? sourceSlug}</div>
					{/each}
				</div>
			{/if}

			{#if error}<p class="error">{error}</p>{/if}
		{/snippet}
		{#snippet footer()}
			<span class="foothint">{t('workshop.opensEditorHint')}</span>
			<div class="sp"></div>
			<Button variant="ghost" size="sm" onclick={close}>{t('action.cancel')}</Button>
			<Button variant="primary" size="sm" onclick={submit} disabled={creating}>
				{creating ? t('editor.saving') : t('action.create')}
			</Button>
		{/snippet}
	</Modal>
{/if}

<style>
	.flabel {
		display: block;
		font-size: 10.5px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--faint);
		margin: 0 0 6px;
	}
	.typegrid-label {
		margin-top: 18px;
	}
	.tin {
		width: 100%;
		height: 36px;
		padding: 0 11px;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		color: var(--text);
		font-family: inherit;
		font-size: 15px;
		outline: none;
	}
	.tin:focus {
		border-color: var(--accent);
		box-shadow: 0 0 0 3px var(--accent-soft);
	}
	.slugprev {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--muted);
		margin: 7px 0 0;
	}
	.slugprev b {
		color: var(--text);
	}
	.typegrid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 8px;
		margin-top: 6px;
	}
	.tcard {
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 11px 12px;
		cursor: pointer;
		background: var(--surface);
		text-align: left;
		font: inherit;
	}
	.tcard:hover {
		border-color: var(--border-strong);
	}
	.tcard.sel {
		border-color: var(--accent);
		background: var(--accent-soft);
		box-shadow: 0 0 0 1px var(--accent);
	}
	.tcard .tn {
		font-weight: 600;
		font-size: 13px;
	}
	.tcard .tf {
		font-family: var(--font-mono);
		font-size: 10.5px;
		color: var(--faint);
		margin-top: 2px;
	}
	.tcard.sel .tf {
		color: var(--accent);
	}
	.resolvebox {
		margin-top: 16px;
		border: 1px solid var(--border);
		border-radius: var(--radius);
		overflow: hidden;
	}
	.resolvebox .rh {
		padding: 9px 12px;
		background: var(--surface-2);
		font-size: 12px;
		color: var(--muted);
		border-bottom: 1px solid var(--border);
	}
	.resolvebox .rh b {
		color: var(--accent);
	}
	.rl {
		padding: 8px 12px;
		border-top: 1px solid var(--border);
		font-size: 13px;
	}
	.rl:first-child {
		border-top: 0;
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
