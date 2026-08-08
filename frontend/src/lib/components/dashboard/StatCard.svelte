<script lang="ts">
	// Stat card for dashboard metrics (docs/ui/screens.md §1). Variants: default
	// (surface), stub (stub tint), danger (danger value color). Clickable when
	// `href` is set. Utilities contained here per ADR 0013.

	let {
		value,
		label,
		variant = 'default',
		suffix = undefined,
		href = undefined
	}: {
		value: number | string;
		label: string;
		variant?: 'default' | 'stub' | 'danger';
		suffix?: string;
		href?: string;
	} = $props();

	const Tag = href ? 'a' : 'div';
</script>

{#if href}
	<a class="stat {variant}" {href}>
		<span class="value">
			{value}{#if suffix}<small>{suffix}</small>{/if}
		</span>
		<span class="label">{label}</span>
	</a>
{:else}
	<div class="stat {variant}">
		<span class="value">
			{value}{#if suffix}<small>{suffix}</small>{/if}
		</span>
		<span class="label">{label}</span>
	</div>
{/if}

<style>
	.stat {
		display: flex;
		flex-direction: column;
		padding: 14px 15px;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		text-decoration: none;
		transition:
			border-color 0.12s ease,
			background 0.12s ease;
	}
	a.stat {
		cursor: pointer;
	}
	a.stat:hover {
		border-color: var(--border-strong);
	}
	.value {
		font-size: 26px;
		font-weight: 700;
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
		letter-spacing: -0.02em;
		color: var(--text);
	}
	.value small {
		font-size: 15px;
		font-weight: 500;
		color: var(--muted);
		margin-left: 2px;
	}
	.label {
		font-size: 12.5px;
		color: var(--muted);
		margin-top: 2px;
	}

	/* Stub variant */
	.stat.stub {
		background: var(--stub-soft);
		border-color: var(--stub-line);
	}
	.stat.stub .value,
	.stat.stub .label {
		color: var(--stub);
	}

	/* Danger variant — only value colored */
	.stat.danger .value {
		color: var(--danger);
	}
</style>
