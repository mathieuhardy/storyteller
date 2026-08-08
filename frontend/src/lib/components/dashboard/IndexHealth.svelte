<script lang="ts">
	// Index health indicator (docs/ui/screens.md §1). Shows watcher status with
	// animated pulse dot. Respects prefers-reduced-motion. Explains external
	// editing capability.
	import { t } from '$i18n/index.svelte';

	let {
		isActive = true
	}: {
		isActive?: boolean;
	} = $props();
</script>

<div class="card">
	<header class="header">
		<h2 class="header-title">{t('dashboard.indexHealth')}</h2>
	</header>

	<div class="content">
		<div class="status" class:active={isActive}>
			<span class="pulse-dot"></span>
			<span class="status-text">{t('dashboard.watcherActive')}</span>
		</div>

		<p class="hint">{t('dashboard.externalEditHint')}</p>
	</div>
</div>

<style>
	.card {
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		overflow: hidden;
	}

	.header {
		padding: 12px 15px;
		border-bottom: 1px solid var(--border);
	}
	.header-title {
		margin: 0;
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--muted);
	}

	.content {
		padding: 15px;
	}

	.status {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 12px;
	}

	.pulse-dot {
		width: 8px;
		height: 8px;
		background: var(--muted);
		border-radius: 50%;
	}

	.status.active .pulse-dot {
		background: var(--ok);
		animation: pulse 2.2s infinite;
	}

	.status.active .status-text {
		color: var(--ok);
		font-weight: 500;
	}

	.status-text {
		font-size: 12.5px;
		color: var(--muted);
	}

	.hint {
		margin: 0;
		padding-top: 12px;
		border-top: 1px solid var(--border);
		font-size: 12px;
		color: var(--muted);
		line-height: 1.5;
	}

	@keyframes pulse {
		0% {
			box-shadow: 0 0 0 0 var(--ok);
		}
		50% {
			box-shadow: 0 0 0 7px transparent;
		}
		100% {
			box-shadow: 0 0 0 0 transparent;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.status.active .pulse-dot {
			animation: none;
		}
	}
</style>
