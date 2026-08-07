<script lang="ts">
	// EN/FR switch in the topbar (docs/ui/i18n.md). A native select keeps it
	// accessible and compact; the choice persists as a local preference.
	import { getLocale, setLocale, t, type Locale } from '$i18n/index.svelte';

	function onchange(event: Event) {
		setLocale((event.currentTarget as HTMLSelectElement).value as Locale);
	}
</script>

<label class="lang" title={t('lang.label')}>
	<span class="sr-only">{t('lang.label')}</span>
	<select value={getLocale()} {onchange} aria-label={t('lang.label')}>
		<option value="en">EN</option>
		<option value="fr">FR</option>
	</select>
</label>

<style>
	.lang select {
		height: 30px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--text);
		font: inherit;
		font-size: 12.5px;
		font-weight: 600;
		padding: 0 6px;
		cursor: pointer;
	}
	.lang select:hover {
		border-color: var(--border-strong);
	}
	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip: rect(0 0 0 0);
	}
</style>
