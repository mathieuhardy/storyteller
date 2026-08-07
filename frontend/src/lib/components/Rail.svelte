<script lang="ts">
	// Collapsible right panel (docs/ui/layout.md §2.4). Hidden by default, opened
	// from the topbar "Links" toggle. On an entry it will host backlinks, outgoing
	// links and metadata; for now it explains what will live here.
	import Icon from './Icon.svelte';
	import IconButton from './IconButton.svelte';
	import { t } from '$i18n/index.svelte';

	let { onClose }: { onClose?: () => void } = $props();
</script>

<aside class="rail" aria-label={t('links.toggle')}>
	<div class="head">
		<span class="title">{t('links.toggle')}</span>
		<IconButton label={t('action.cancel')} onclick={() => onClose?.()}>
			<Icon name="x" size={15} />
		</IconButton>
	</div>
	<div class="body">
		<p class="hint">{t('screen.comingSoonBody')}</p>
	</div>
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
	}
	.title {
		font-weight: 600;
		font-size: 13px;
	}
	.body {
		padding: 14px;
		overflow-y: auto;
	}
	.hint {
		margin: 0;
		color: var(--faint);
		font-size: 12.5px;
		line-height: 1.6;
	}
</style>
