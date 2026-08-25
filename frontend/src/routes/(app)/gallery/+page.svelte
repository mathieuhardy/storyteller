<script lang="ts">
	// Media gallery screen (docs/ui/screens.md §6): visual browsing of every
	// file under assets/. `GET /assets` has no server-side filter, so the
	// filename filter is client-side — same reasoning as the Chantier's
	// client-side aggregation, fine at the local/single-user scale this app
	// targets (docs/principles.md).
	import { t } from '$i18n/index.svelte';
	import { uploadAsset, assetUrl, ApiError } from '$api/client';
	import Icon from '$components/Icon.svelte';
	import Button from '$components/Button.svelte';
	import Modal from '$components/Modal.svelte';
	import type { AssetInfo } from '$api/types';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	let filter = $state('');
	let fileInput: HTMLInputElement | undefined = $state();
	let uploading = $state(false);
	let uploadError = $state<string | null>(null);
	let detail = $state<AssetInfo | null>(null);
	let copied = $state(false);

	const filtered = $derived(
		filter.trim()
			? data.assets.filter((a) => a.path.toLowerCase().includes(filter.trim().toLowerCase()))
			: data.assets
	);

	function formatSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}

	async function onFileChosen(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		input.value = '';
		if (!file) return;

		uploading = true;
		uploadError = null;
		try {
			await uploadAsset(file, file.name);
		} catch (e) {
			uploadError = e instanceof ApiError ? e.message : String(e);
		} finally {
			uploading = false;
		}
	}

	function openDetail(asset: AssetInfo) {
		detail = asset;
		copied = false;
	}

	async function copyLink() {
		if (!detail) return;
		try {
			await navigator.clipboard.writeText(`![[${detail.path}]]`);
			copied = true;
		} catch {
			// Clipboard access can be denied by the browser; nothing to recover.
		}
	}
</script>

<div class="screen">
	<div class="head">
		<h1><Icon name="image" size={19} />{t('gallery.title')}</h1>
		<p class="sub">{t('gallery.subtitle')}</p>
	</div>

	<div class="toolbar">
		<input
			class="filter"
			type="search"
			bind:value={filter}
			placeholder={t('gallery.filterPlaceholder')}
		/>
		<Button variant="primary" size="sm" onclick={() => fileInput?.click()} disabled={uploading}>
			<Icon name="plus" size={15} />
			{uploading ? t('gallery.uploading') : t('gallery.upload')}
		</Button>
		<input bind:this={fileInput} type="file" class="hidden-input" onchange={onFileChosen} />
	</div>

	{#if uploadError}<p class="upload-error">{uploadError}</p>{/if}

	{#if data.assets.length === 0}
		<p class="empty">{t('gallery.empty')}</p>
	{:else if filtered.length === 0}
		<div class="empty">
			<p>{t('gallery.noResults', { query: filter })}</p>
			<button class="clear" onclick={() => (filter = '')}>{t('gallery.clearFilter')}</button>
		</div>
	{:else}
		<div class="grid">
			{#each filtered as asset (asset.path)}
				<button class="card" onclick={() => openDetail(asset)}>
					{#if asset.kind === 'image'}
						<img class="thumb" src={assetUrl(asset.path)} alt="" loading="lazy" />
					{:else}
						<div class="filetile"><Icon name="file" size={26} /></div>
					{/if}
					<span class="cname" title={asset.path}>{asset.path}</span>
				</button>
			{/each}
		</div>
	{/if}
</div>

<Modal
	open={detail !== null}
	kicker={detail?.kind === 'image' ? t('gallery.kindImage') : t('gallery.kindFile')}
	title={detail?.path ?? ''}
	onclose={() => (detail = null)}
>
	{#snippet body()}
		{#if detail}
			<div class="detail">
				{#if detail.kind === 'image'}
					<img class="preview" src={assetUrl(detail.path)} alt="" />
				{:else}
					<div class="filetile lg"><Icon name="file" size={40} /></div>
				{/if}
				<dl class="meta">
					<dt>{t('gallery.path')}</dt>
					<dd class="mono">{detail.path}</dd>
					<dt>{t('gallery.size')}</dt>
					<dd>{formatSize(detail.size)}</dd>
				</dl>
			</div>
		{/if}
	{/snippet}
	{#snippet footer()}
		<Button variant="primary" size="sm" onclick={copyLink}>
			{copied ? t('gallery.copied') : t('gallery.copyLink')}
		</Button>
	{/snippet}
</Modal>

<style>
	.screen {
		max-width: 1040px;
		margin: 0 auto;
		padding: 22px 26px 60px;
	}
	.head h1 {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 21px;
		letter-spacing: -0.02em;
		margin: 0;
		font-weight: 660;
	}
	.head .sub {
		color: var(--muted);
		font-size: 13px;
		margin-top: 6px;
		max-width: 62ch;
	}
	.toolbar {
		display: flex;
		align-items: center;
		gap: 10px;
		margin: 20px 0 18px;
	}
	.filter {
		flex: 1;
		max-width: 320px;
		height: 32px;
		padding: 0 11px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--text);
		font: inherit;
		font-size: 13px;
	}
	.filter:focus {
		outline: none;
		border-color: var(--accent);
		box-shadow: 0 0 0 3px var(--accent-soft);
	}
	.hidden-input {
		display: none;
	}
	.upload-error {
		color: var(--danger);
		font-size: 12.5px;
		margin: -8px 0 14px;
	}
	.empty {
		color: var(--muted);
		font-size: 13.5px;
		padding: 40px 0;
		text-align: center;
	}
	.clear {
		margin-top: 8px;
		border: 0;
		background: transparent;
		color: var(--accent);
		font: inherit;
		font-size: 13px;
		cursor: pointer;
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
		gap: 12px;
	}
	.card {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 8px;
		border: 1px solid var(--border);
		border-radius: var(--radius);
		background: var(--surface);
		cursor: pointer;
		text-align: left;
		font: inherit;
	}
	.card:hover {
		border-color: var(--border-strong);
		box-shadow: var(--shadow);
	}
	.thumb {
		width: 100%;
		aspect-ratio: 1;
		object-fit: cover;
		border-radius: var(--radius-sm);
		display: block;
		background: var(--surface-2);
	}
	.filetile {
		width: 100%;
		aspect-ratio: 1;
		display: grid;
		place-items: center;
		border-radius: var(--radius-sm);
		background: var(--surface-2);
		color: var(--faint);
	}
	.filetile.lg {
		aspect-ratio: 4 / 3;
	}
	.cname {
		font-size: 11.5px;
		color: var(--muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.detail {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.preview {
		width: 100%;
		max-height: 320px;
		object-fit: contain;
		border-radius: var(--radius-sm);
		background: var(--surface-2);
	}
	.meta {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 4px 12px;
		margin: 0;
		font-size: 13px;
	}
	.meta dt {
		color: var(--faint);
	}
	.meta dd {
		margin: 0;
		color: var(--text);
		word-break: break-all;
	}
	.meta dd.mono {
		font-family: var(--font-mono);
		font-size: 12px;
	}
</style>
