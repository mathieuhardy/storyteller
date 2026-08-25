<script lang="ts">
	// Link graph screen (docs/ui/screens.md §7): every entry as a node, every
	// resolved link as an edge, laid out client-side (ADR 0018). Hover
	// highlights a node's direct neighbors; click navigates to the entry;
	// wheel/drag zoom and pan the SVG viewBox.
	import { goto } from '$app/navigation';
	import { t, typeLabel } from '$i18n/index.svelte';
	import Icon from '$components/Icon.svelte';
	import { computeLayout } from '$lib/graph/layout';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const WIDTH = 1200;
	const HEIGHT = 800;

	const positions = $derived(computeLayout(data.graph.nodes, data.graph.edges, {
		width: WIDTH,
		height: HEIGHT
	}));

	const neighbors = $derived.by(() => {
		const map = new Map<string, Set<string>>();
		for (const node of data.graph.nodes) map.set(node.slug, new Set());
		for (const edge of data.graph.edges) {
			map.get(edge.source)?.add(edge.target);
			map.get(edge.target)?.add(edge.source);
		}
		return map;
	});

	const degree = $derived.by(() => {
		const map = new Map<string, number>();
		for (const [slug, set] of neighbors) map.set(slug, set.size);
		return map;
	});

	let hovered = $state<string | null>(null);
	const activeNeighbors = $derived(hovered ? (neighbors.get(hovered) ?? new Set()) : null);

	let svgEl: SVGSVGElement | undefined = $state();
	let viewBox = $state({ x: 0, y: 0, w: WIDTH, h: HEIGHT });
	let dragging = false;
	let lastPointer = { x: 0, y: 0 };

	function nodeRadius(slug: string): number {
		return 6 + Math.min(degree.get(slug) ?? 0, 12);
	}

	function onWheel(event: WheelEvent) {
		event.preventDefault();
		if (!svgEl) return;
		const rect = svgEl.getBoundingClientRect();
		const mx = (event.clientX - rect.left) / rect.width;
		const my = (event.clientY - rect.top) / rect.height;
		const factor = event.deltaY > 0 ? 1.12 : 1 / 1.12;
		const newW = Math.min(WIDTH * 4, Math.max(WIDTH / 8, viewBox.w * factor));
		const newH = Math.min(HEIGHT * 4, Math.max(HEIGHT / 8, viewBox.h * factor));
		viewBox = {
			x: viewBox.x + (viewBox.w - newW) * mx,
			y: viewBox.y + (viewBox.h - newH) * my,
			w: newW,
			h: newH
		};
	}

	function onPointerDown(event: PointerEvent) {
		dragging = true;
		lastPointer = { x: event.clientX, y: event.clientY };
		(event.target as Element).setPointerCapture(event.pointerId);
	}

	function onPointerMove(event: PointerEvent) {
		if (!dragging || !svgEl) return;
		const rect = svgEl.getBoundingClientRect();
		const dx = ((event.clientX - lastPointer.x) / rect.width) * viewBox.w;
		const dy = ((event.clientY - lastPointer.y) / rect.height) * viewBox.h;
		viewBox = { ...viewBox, x: viewBox.x - dx, y: viewBox.y - dy };
		lastPointer = { x: event.clientX, y: event.clientY };
	}

	function onPointerUp() {
		dragging = false;
	}

	function openEntry(slug: string) {
		goto(`/entry/${slug}`);
	}
</script>

<div class="screen">
	<div class="head">
		<h1><Icon name="network" size={19} />{t('graph.title')}</h1>
		<p class="sub">{t('graph.subtitle')}</p>
	</div>

	{#if data.graph.nodes.length === 0}
		<p class="empty">{t('graph.empty')}</p>
	{:else}
		<p class="count">
			{t('graph.nodeCount', { count: data.graph.nodes.length, edges: data.graph.edges.length })}
		</p>
		<div class="canvas">
			<svg
				bind:this={svgEl}
				viewBox="{viewBox.x} {viewBox.y} {viewBox.w} {viewBox.h}"
				onwheel={onWheel}
				onpointerdown={onPointerDown}
				onpointermove={onPointerMove}
				onpointerup={onPointerUp}
				onpointerleave={onPointerUp}
				role="img"
				aria-label={t('graph.title')}
			>
				<g class="edges">
					{#each data.graph.edges as edge (edge.source + '→' + edge.target)}
						{@const a = positions.get(edge.source)}
						{@const b = positions.get(edge.target)}
						{#if a && b}
							<line
								x1={a.x}
								y1={a.y}
								x2={b.x}
								y2={b.y}
								class:dim={hovered !== null &&
									hovered !== edge.source &&
									hovered !== edge.target}
								class:lit={hovered === edge.source || hovered === edge.target}
							/>
						{/if}
					{/each}
				</g>
				<g class="nodes">
					{#each data.graph.nodes as node (node.slug)}
						{@const p = positions.get(node.slug)}
						{#if p}
							<g
								class="node"
								class:dim={hovered !== null &&
									hovered !== node.slug &&
									!activeNeighbors?.has(node.slug)}
								class:isolated={(degree.get(node.slug) ?? 0) === 0}
								transform="translate({p.x},{p.y})"
								onpointerenter={() => (hovered = node.slug)}
								onpointerleave={() => (hovered = null)}
								onclick={() => openEntry(node.slug)}
								role="button"
								tabindex="0"
								onkeydown={(e) => e.key === 'Enter' && openEntry(node.slug)}
							>
								<circle r={nodeRadius(node.slug)} class:error={node.has_errors} />
								{#if hovered === node.slug}
									<text class="label" y={-nodeRadius(node.slug) - 6}
										>{node.title} · {typeLabel(node.type)}</text
									>
								{/if}
							</g>
						{/if}
					{/each}
				</g>
			</svg>
		</div>
	{/if}
</div>

<style>
	.screen {
		padding: 20px 26px 30px;
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}
	.head h1 {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 21px;
		letter-spacing: -0.02em;
		margin: 0;
		font-weight: 660;
	}
	.head .sub {
		color: var(--muted);
		font-size: 13px;
		margin-top: 6px;
		max-width: 62ch;
	}
	.count {
		color: var(--faint);
		font-size: 12px;
		margin: 14px 0 8px;
	}
	.empty {
		color: var(--muted);
		font-size: 13.5px;
		padding: 40px 0;
		text-align: center;
	}
	.canvas {
		flex: 1;
		min-height: 420px;
		border: 1px solid var(--border);
		border-radius: var(--radius);
		background: var(--surface);
		overflow: hidden;
	}
	svg {
		width: 100%;
		height: 100%;
		cursor: grab;
		touch-action: none;
	}
	.edges line {
		stroke: var(--border-strong);
		stroke-width: 1.5;
		transition: opacity 0.12s ease;
	}
	.edges line.dim {
		opacity: 0.15;
	}
	.edges line.lit {
		stroke: var(--accent);
		opacity: 0.9;
	}
	.node {
		cursor: pointer;
		transition: opacity 0.12s ease;
	}
	.node.dim {
		opacity: 0.25;
	}
	.node circle {
		fill: var(--accent);
		fill-opacity: 0.85;
		stroke: var(--surface);
		stroke-width: 1.5;
	}
	.node.isolated circle {
		fill: var(--faint);
		fill-opacity: 0.6;
	}
	.node circle.error {
		fill: var(--danger);
	}
	.node .label {
		text-anchor: middle;
		font-size: 12px;
		fill: var(--text);
		paint-order: stroke;
		stroke: var(--surface);
		stroke-width: 3px;
		stroke-linejoin: round;
	}
</style>
