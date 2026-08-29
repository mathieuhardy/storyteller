<script lang="ts">
	// Live word count display for book mode.
	import { t, formatNumber } from '$i18n/index.svelte';

	let { count }: { count: number } = $props();
</script>

<span class="word-count mono">
	{formatNumber(count)} {t('book.words')}
</span>

<style>
	.word-count {
		font-size: 12px;
		color: var(--muted);
	}
</style>
