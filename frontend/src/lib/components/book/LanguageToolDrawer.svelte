<script lang="ts">
	// Right-side drawer showing LanguageTool grammar/spelling suggestions.
	// Displays matches from the current document with replacement options.
	import { t } from '$i18n/index.svelte';
	import Icon from '$components/Icon.svelte';
	import type { LTMatch } from '$api/client';

	let {
		open = false,
		matches = [],
		isChecking = false,
		onClose,
		onCheck,
		onApplyFix
	}: {
		open?: boolean;
		matches?: LTMatch[];
		isChecking?: boolean;
		onClose?: () => void;
		onCheck?: () => void;
		onApplyFix?: (match: LTMatch, replacement: string) => void;
	} = $props();
</script>

{#if open}
	<aside class="drawer">
		<header class="drawer-header">
			<h2>{t('book.ltDrawerTitle')}</h2>
			<div class="header-actions">
				<button
					class="check-btn"
					onclick={() => onCheck?.()}
					disabled={isChecking}
					title={t('book.ltCheckDocument')}
				>
					{#if isChecking}
						<span class="spinner"></span>
						{t('book.ltChecking')}
					{:else}
						<Icon name="spell-check" size={14} />
						{t('book.ltCheckDocument')}
					{/if}
				</button>
				<button class="close-btn" onclick={() => onClose?.()} title={t('action.close')}>
					<Icon name="x" size={16} />
				</button>
			</div>
		</header>

		<div class="drawer-content">
			{#if isChecking}
				<div class="empty-state">
					<span class="spinner large"></span>
					<p>{t('book.ltChecking')}</p>
				</div>
			{:else if matches.length === 0}
				<div class="empty-state">
					<Icon name="check" size={32} />
					<p>{t('book.ltNoIssues')}</p>
				</div>
			{:else}
				<p class="match-count">{t('book.ltIssueCount', { count: matches.length })}</p>
				<ul class="matches">
					{#each matches as match, i (i)}
						<li class="match">
							<div class="match-header">
								<span class="rule-id">{match.rule.id}</span>
							</div>
							<p class="message">{match.message}</p>
							{#if match.context}
								<p class="context">
									<span class="context-before">{match.context.text.slice(0, match.context.offset)}</span><mark>{match.context.text.slice(match.context.offset, match.context.offset + match.context.length)}</mark><span class="context-after">{match.context.text.slice(match.context.offset + match.context.length)}</span>
								</p>
							{/if}
							{#if match.replacements.length > 0}
								<div class="replacements">
									<span class="replacements-label">{t('book.ltSuggestions')}</span>
									<div class="replacement-buttons">
										{#each match.replacements.slice(0, 5) as replacement}
											<button
												class="replacement-btn"
												onclick={() => onApplyFix?.(match, replacement.value)}
											>
												{replacement.value || t('book.ltRemove')}
											</button>
										{/each}
									</div>
								</div>
							{/if}
						</li>
					{/each}
				</ul>
			{/if}
		</div>
	</aside>
{/if}

<style>
	.drawer {
		position: fixed;
		top: 48px;
		right: 0;
		bottom: 0;
		width: 340px;
		background: var(--surface);
		border-left: 1px solid var(--border);
		display: flex;
		flex-direction: column;
		z-index: 30;
		box-shadow: -4px 0 12px rgba(0, 0, 0, 0.1);
	}

	.drawer-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		border-bottom: 1px solid var(--border);
		gap: 8px;
	}

	.drawer-header h2 {
		margin: 0;
		font-size: 14px;
		font-weight: 600;
		color: var(--text);
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.check-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 5px 10px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--surface-2);
		color: var(--text);
		font: inherit;
		font-size: 11px;
		cursor: pointer;
	}

	.check-btn:hover:not(:disabled) {
		background: var(--surface-3);
	}

	.check-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: 0;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--muted);
		cursor: pointer;
	}

	.close-btn:hover {
		background: var(--surface-2);
		color: var(--text);
	}

	.drawer-content {
		flex: 1;
		overflow-y: auto;
		padding: 12px;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 200px;
		color: var(--muted);
		gap: 12px;
	}

	.empty-state p {
		margin: 0;
		font-size: 13px;
	}

	.match-count {
		margin: 0 0 12px;
		font-size: 11px;
		color: var(--muted);
	}

	.matches {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.match {
		padding: 10px;
		background: var(--surface-2);
		border-radius: var(--radius);
		border: 1px solid var(--border);
	}

	.match-header {
		margin-bottom: 6px;
	}

	.rule-id {
		font-size: 10px;
		font-weight: 500;
		color: var(--muted);
		text-transform: uppercase;
		letter-spacing: 0.03em;
	}

	.message {
		margin: 0 0 8px;
		font-size: 12px;
		color: var(--text);
		line-height: 1.4;
	}

	.context {
		margin: 0 0 8px;
		padding: 6px 8px;
		background: var(--surface);
		border-radius: var(--radius-sm);
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--muted);
		white-space: pre-wrap;
		word-wrap: break-word;
		overflow-wrap: break-word;
	}

	.context mark {
		background: var(--warning-bg, rgba(234, 179, 8, 0.2));
		color: var(--warning);
		padding: 1px 2px;
		border-radius: 2px;
	}

	.replacements {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.replacements-label {
		font-size: 10px;
		color: var(--muted);
		text-transform: uppercase;
		letter-spacing: 0.03em;
	}

	.replacement-buttons {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}

	.replacement-btn {
		padding: 4px 8px;
		border: 1px solid var(--accent);
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--accent);
		font: inherit;
		font-size: 11px;
		cursor: pointer;
	}

	.replacement-btn:hover {
		background: var(--accent);
		color: var(--accent-fg);
	}

	.spinner {
		display: inline-block;
		width: 12px;
		height: 12px;
		border: 2px solid var(--border);
		border-top-color: var(--accent);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	.spinner.large {
		width: 24px;
		height: 24px;
		border-width: 3px;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	@media (max-width: 768px) {
		.drawer {
			width: 100%;
			left: 0;
		}
	}
</style>
