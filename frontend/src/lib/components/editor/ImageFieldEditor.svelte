<script lang="ts">
	// Image / image-list field (docs/ui/components.md §4): thumbnails of the
	// current path(s) plus an upload button (`POST /assets`). A single `image`
	// field keeps 0-or-1 path; `image-list` keeps several — same uniform
	// array contract as link fields (see `EntryEditor.svelte`).
	import { uploadAsset, assetUrl, ApiError } from '$api/client';
	import { t } from '$i18n/index.svelte';

	let {
		paths = $bindable([]),
		multi
	}: {
		paths?: string[];
		multi: boolean;
	} = $props();

	let fileInput: HTMLInputElement | undefined = $state();
	let uploading = $state(false);
	let error = $state<string | null>(null);

	function remove(index: number) {
		paths = paths.filter((_, i) => i !== index);
	}

	async function onFileChosen(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		input.value = '';
		if (!file) return;

		uploading = true;
		error = null;
		try {
			const { path } = await uploadAsset(file, file.name);
			paths = multi ? [...paths, path] : [path];
		} catch (e) {
			error = e instanceof ApiError ? e.message : String(e);
		} finally {
			uploading = false;
		}
	}
</script>

<div class="image-field">
	{#each paths as path, i (path)}
		<div class="thumb">
			<img src={assetUrl(path)} alt="" />
			<button type="button" class="remove" onclick={() => remove(i)} aria-label="remove"
				>✕</button
			>
		</div>
	{/each}

	{#if multi || paths.length === 0}
		<button
			type="button"
			class="upload-btn"
			onclick={() => fileInput?.click()}
			disabled={uploading}
		>
			{uploading ? t('editor.uploading') : t('editor.uploadImage')}
		</button>
	{/if}
	<input
		bind:this={fileInput}
		type="file"
		accept="image/*"
		class="hidden-input"
		onchange={onFileChosen}
	/>

	{#if error}<span class="error">{error}</span>{/if}
</div>

<style>
	.image-field {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}
	.thumb {
		position: relative;
		width: 56px;
		height: 56px;
		border-radius: var(--radius-sm);
		overflow: hidden;
		border: 1px solid var(--border);
		flex: none;
	}
	.thumb img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}
	.thumb .remove {
		position: absolute;
		top: 2px;
		right: 2px;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		border: 0;
		background: rgba(0, 0, 0, 0.6);
		color: #fff;
		font-size: 10px;
		cursor: pointer;
		display: grid;
		place-items: center;
	}
	.upload-btn {
		height: 34px;
		padding: 0 12px;
		border: 1px dashed var(--border-strong);
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--muted);
		font-size: 12.5px;
		font: inherit;
		cursor: pointer;
	}
	.upload-btn:hover {
		border-color: var(--accent);
		color: var(--accent);
	}
	.upload-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
	.hidden-input {
		display: none;
	}
	.error {
		color: var(--danger);
		font-size: 12px;
	}
</style>
