// Ubicación: `apps/web/src/lib/api/sedes.ts`
//
// Descripción: API module para operaciones CRUD con sedes regionales.
//              Proporciona listSedes, getSede, createSede, updateSede.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional)

import { apiClient } from './client';
import type { SedeResponse, CreateSedeRequest, UpdateSedeRequest } from '$lib/generated/api-types';

export async function listSedes(): Promise<SedeResponse[]> {
	return apiClient.get<SedeResponse[]>('/api/v1/sedes');
}

export async function getSede(id: string): Promise<SedeResponse> {
	return apiClient.get<SedeResponse>(`/api/v1/sedes/${id}`);
}

export async function createSede(data: CreateSedeRequest): Promise<SedeResponse> {
	return apiClient.post<SedeResponse>('/api/v1/sedes', data);
}

export async function updateSede(id: string, data: UpdateSedeRequest): Promise<SedeResponse> {
	return apiClient.put<SedeResponse>(`/api/v1/sedes/${id}`, data);
}
