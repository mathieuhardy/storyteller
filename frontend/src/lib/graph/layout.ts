// Force-directed layout for the link graph (docs/ui/screens.md §7,
// ADR 0018). A hand-rolled Fruchterman-Reingold simulation — no charting/graph
// library dependency, see the ADR for why. Pure function: nodes/edges in,
// positions out: easy to reason about and to swap out later.

export interface LayoutPoint {
	x: number;
	y: number;
}

export interface LayoutOptions {
	width?: number;
	height?: number;
	iterations?: number;
}

/** Positions every node inside `width`×`height`, pulling connected nodes
 * closer together and pushing every pair apart otherwise. Deterministic given
 * the same random seed is out of scope here — the graph does not need to be
 * pixel-stable across reloads, only readable. */
export function computeLayout(
	nodes: { slug: string }[],
	edges: { source: string; target: string }[],
	opts: LayoutOptions = {}
): Map<string, LayoutPoint> {
	const width = opts.width ?? 1000;
	const height = opts.height ?? 1000;
	const iterations = opts.iterations ?? 200;

	const positions = new Map<string, LayoutPoint>();
	if (nodes.length === 0) return positions;

	// Ideal edge length: the frame area shared evenly among nodes.
	const k = Math.sqrt((width * height) / nodes.length);

	for (const node of nodes) {
		const angle = Math.random() * Math.PI * 2;
		const radius = (Math.min(width, height) / 3) * Math.sqrt(Math.random());
		positions.set(node.slug, {
			x: width / 2 + Math.cos(angle) * radius,
			y: height / 2 + Math.sin(angle) * radius
		});
	}

	const realEdges = edges.filter(
		(e) => positions.has(e.source) && positions.has(e.target) && e.source !== e.target
	);

	let temperature = Math.min(width, height) / 10;
	const cooling = temperature / iterations;

	for (let iter = 0; iter < iterations; iter++) {
		const displacement = new Map<string, LayoutPoint>(nodes.map((n) => [n.slug, { x: 0, y: 0 }]));

		// Every pair repels — O(n²), fine at the local/single-project scale this
		// app targets (a few hundred entries at most).
		for (let i = 0; i < nodes.length; i++) {
			for (let j = i + 1; j < nodes.length; j++) {
				const a = nodes[i].slug;
				const b = nodes[j].slug;
				const pa = positions.get(a)!;
				const pb = positions.get(b)!;
				let dx = pa.x - pb.x;
				let dy = pa.y - pb.y;
				const dist = Math.hypot(dx, dy) || 0.01;
				const force = (k * k) / dist;
				dx = (dx / dist) * force;
				dy = (dy / dist) * force;
				const da = displacement.get(a)!;
				da.x += dx;
				da.y += dy;
				const db = displacement.get(b)!;
				db.x -= dx;
				db.y -= dy;
			}
		}

		// Edges attract their two endpoints.
		for (const edge of realEdges) {
			const pa = positions.get(edge.source)!;
			const pb = positions.get(edge.target)!;
			let dx = pa.x - pb.x;
			let dy = pa.y - pb.y;
			const dist = Math.hypot(dx, dy) || 0.01;
			const force = (dist * dist) / k;
			dx = (dx / dist) * force;
			dy = (dy / dist) * force;
			const da = displacement.get(edge.source)!;
			da.x -= dx;
			da.y -= dy;
			const db = displacement.get(edge.target)!;
			db.x += dx;
			db.y += dy;
		}

		// Apply, capped by the cooling "temperature" (simulated annealing), and
		// clamped inside the frame.
		for (const node of nodes) {
			const d = displacement.get(node.slug)!;
			const dist = Math.hypot(d.x, d.y) || 0.01;
			const capped = Math.min(dist, temperature);
			const p = positions.get(node.slug)!;
			p.x = Math.min(width, Math.max(0, p.x + (d.x / dist) * capped));
			p.y = Math.min(height, Math.max(0, p.y + (d.y / dist) * capped));
		}

		temperature = Math.max(temperature - cooling, 0.01);
	}

	return positions;
}
