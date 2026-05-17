// Ubicación: `apps/web/src/lib/api/intrusions.ts`
//
// Descripción: API module para gestión de eventos de intrusión detectados.
//              listIntrusions, getIntrusion, resolveIntrusion, markFalsePositive.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional)

import { apiClient } from './client';
import type {
	IntrusionEvent,
	CreateIntrusionRequest,
} from '$lib/generated/api-types';

export async function listIntrusions(params?: {
	status?: string;
	limit?: number;
}): Promise<IntrusionEvent[]> {
	const searchParams = new URLSearchParams();
	if (params?.status) searchParams.set('status', params.status);
	if (params?.limit) searchParams.set('limit', params.limit.toString());

	const query = searchParams.toString();
	return apiClient.get<IntrusionEvent[]>(`/api/v1/intrusions${query ? `?${query}` : ''}`);
}

export async function getIntrusion(id: string): Promise<IntrusionEvent> {
	return apiClient.get<IntrusionEvent>(`/api/v1/intrusions/${id}`);
}

export async function resolveIntrusion(id: string): Promise<IntrusionEvent> {
	return apiClient.put<IntrusionEvent>(`/api/v1/intrusions/${id}/resolve`);
}

export async function markFalsePositive(id: string): Promise<IntrusionEvent> {
	return apiClient.put<IntrusionEvent>(`/api/v1/intrusions/${id}/false-positive`);
}
