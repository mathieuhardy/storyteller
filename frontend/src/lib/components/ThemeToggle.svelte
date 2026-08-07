<script lang="ts">
	// Cycles system → light → dark, showing the icon of the current choice.
	import Icon from './Icon.svelte';
	import IconButton from './IconButton.svelte';
	import { cycleTheme, getTheme } from '$lib/stores/theme.svelte';
	import { t } from '$i18n/index.svelte';

	const icon = $derived(
		getTheme() === 'light' ? 'sun' : getTheme() === 'dark' ? 'moon' : 'monitor'
	);
	const label = $derived(
		`${t('theme.label')}: ${t(getTheme() === 'light' ? 'theme.light' : getTheme() === 'dark' ? 'theme.dark' : 'theme.system')}`
	);
</script>

<IconButton {label} onclick={cycleTheme}>
	<Icon name={icon} />
</IconButton>
