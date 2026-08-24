<script lang="ts">
	// Sticky save bar (docs/ui/components.md §4): dirty reminder, non-destructive
	// note, ⌘S hint, Cancel/Save actions.
	import Button from '$components/Button.svelte';
	import { t } from '$i18n/index.svelte';

	let {
		dirty,
		saving,
		error = null,
		oncancel,
		onsave
	}: {
		dirty: boolean;
		saving: boolean;
		error?: string | null;
		oncancel: () => void;
		onsave: () => void;
	} = $props();
</script>

<div class="savebar">
	<span class="msg">
		{#if error}
			<span class="error">{error}</span>
		{:else if dirty}
			<b>{t('editor.unsaved')}</b> — {t('editor.nonDestructiveHint')}
		{:else}
			{t('editor.noChanges')}
		{/if}
	</span>
	<div class="sp"></div>
	<span class="kbd">⌘S</span>
	<Button variant="ghost" size="sm" onclick={oncancel}>{t('action.cancel')}</Button>
	<Button variant="primary" size="sm" onclick={onsave} disabled={saving}>
		{saving ? t('editor.saving') : t('action.save')}
	</Button>
</div>

<style>
	.savebar {
		position: sticky;
		bottom: 0;
		margin: 26px 0 0;
		padding: 12px 0;
		background: color-mix(in srgb, var(--surface) 88%, transparent);
		backdrop-filter: blur(8px);
		border-top: 1px solid var(--border);
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.msg {
		font-size: 12.5px;
		color: var(--muted);
	}
	.msg b {
		color: var(--warn);
	}
	.error {
		color: var(--danger);
	}
	.sp {
		flex: 1;
	}
	.kbd {
		font-family: var(--font-mono);
		font-size: 11px;
		background: var(--surface-2);
		border: 1px solid var(--border);
		border-radius: 4px;
		padding: 1px 5px;
		color: var(--muted);
	}
</style>
