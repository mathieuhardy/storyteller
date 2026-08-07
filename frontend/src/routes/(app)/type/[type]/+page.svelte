<script lang="ts">
	import { page } from '$app/state';
	import Placeholder from '$components/Placeholder.svelte';
	import { typeLabel, t } from '$i18n/index.svelte';
</script>

<Placeholder title={typeLabel(page.params.type ?? '')} body={t('screen.comingSoonBody')} />
