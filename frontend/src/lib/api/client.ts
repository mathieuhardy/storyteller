// Abstract client over the HTTP API (docs/api.md §1). Screens call these
// functions and never build URLs or touch `fetch` directly, so the transport can
// change (a Tauri command layer, M7) without touching the UI. Every call takes
// an optional `fetch` so it works inside SvelteKit `load` (which supplies one)
// and in components (where it defaults to the global).

import type {
	Entry,
	EntrySummary,
	OutgoingLink,
	Backlink,
	Page,
	ProjectResponse,
	ProjectsResponse,
	Stub,
	TypeResponse,
	VersionResponse
} from './types';

/** Base path of the versioned API; same origin (dev proxy / prod server). */
const BASE = '/api/v1';

type Fetch = typeof fetch;

/** A request that the server refused, carrying its normalized error body. */
export class ApiError extends Error {
	constructor(
		readonly status: number,
		readonly code: string,
		message: string
	) {
		super(message);
		this.name = 'ApiError';
	}
}

async function request<T>(
	path: string,
	init: RequestInit | undefined,
	fetchImpl: Fetch
): Promise<T> {
	let response: Response;
	try {
		response = await fetchImpl(`${BASE}${path}`, init);
	} catch (cause) {
		// A dead server / network drop never reaches an HTTP status.
		throw new ApiError(0, 'network_error', `cannot reach the server: ${String(cause)}`);
	}

	const body = await response.text();
	const parsed = body ? JSON.parse(body) : null;

	if (!response.ok) {
		const error = parsed?.error ?? {};
		throw new ApiError(
			response.status,
			error.code ?? 'error',
			error.message ?? response.statusText
		);
	}
	return parsed as T;
}

function json(payload: unknown): RequestInit {
	return {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(payload)
	};
}

// --- Projects -------------------------------------------------------------

export const listProjects = (fetchImpl: Fetch = fetch): Promise<ProjectsResponse> =>
	request('/projects', undefined, fetchImpl);

export const openProject = (path: string, fetchImpl: Fetch = fetch): Promise<ProjectResponse> =>
	request('/projects/open', json({ path }), fetchImpl);

export const getProject = (fetchImpl: Fetch = fetch): Promise<ProjectResponse> =>
	request('/project', undefined, fetchImpl);

export const getVersion = (fetchImpl: Fetch = fetch): Promise<VersionResponse> =>
	request('/version', undefined, fetchImpl);

// --- Types ----------------------------------------------------------------

export const getTypes = (fetchImpl: Fetch = fetch): Promise<TypeResponse[]> =>
	request('/types', undefined, fetchImpl);

export const getType = (type: string, fetchImpl: Fetch = fetch): Promise<TypeResponse> =>
	request(`/types/${encodeURIComponent(type)}`, undefined, fetchImpl);

// --- Entities (wired as the screens land) ---------------------------------

export const getEntities = (
	query = '',
	fetchImpl: Fetch = fetch
): Promise<Page<EntrySummary>> => request(`/entities${query}`, undefined, fetchImpl);

export const getEntity = (
	slug: string,
	opts: { backlinks?: boolean } = {},
	fetchImpl: Fetch = fetch
): Promise<Entry> => {
	const query = opts.backlinks ? '?include=backlinks' : '';
	return request(`/entities/${encodeURIComponent(slug)}${query}`, undefined, fetchImpl);
};

export const getLinks = (slug: string, fetchImpl: Fetch = fetch): Promise<OutgoingLink[]> =>
	request(`/entities/${encodeURIComponent(slug)}/links`, undefined, fetchImpl);

export const getBacklinks = (slug: string, fetchImpl: Fetch = fetch): Promise<Backlink[]> =>
	request(`/entities/${encodeURIComponent(slug)}/backlinks`, undefined, fetchImpl);

export const getStubs = (fetchImpl: Fetch = fetch): Promise<Stub[]> =>
	request('/stubs', undefined, fetchImpl);
