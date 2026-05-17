// Ubicación: `apps/web/src/lib/api/alerts.ts`
//
// Descripción: API module para gestión de alertas del sistema de monitoreo.
//              listAlerts, getAlert, createAlert, acknowledgeAlert, resolveAlert.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional)

import { apiClient } from './client';
import type { AlertResponse, AlertListResponse, CreateAlertRequest } from '$lib/generated/api-types';

export async function listAlerts(): Promise<AlertListResponse> {
	return apiClient.get<AlertListResponse>('/api/v1/alerts');
}

export async function getAlert(id: string): Promise<AlertResponse> {
	return apiClient.get<AlertResponse>(`/api/v1/alerts/${id}`);
}

export async function createAlert(data: CreateAlertRequest): Promise<AlertResponse> {
	return apiClient.post<AlertResponse>('/api/v1/alerts', data);
}

export async function acknowledgeAlert(id: string, userId: string): Promise<AlertResponse> {
	return apiClient.put<AlertResponse>(`/api/v1/alerts/${id}/acknowledge`, { userId });
}

export async function resolveAlert(id: string): Promise<AlertResponse> {
	return apiClient.put<AlertResponse>(`/api/v1/alerts/${id}/resolve`);
}
