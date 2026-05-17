// Ubicación: `apps/web/src/lib/api/topology.ts`
//
// Descripción: API module para obtener topología de red y métricas de dispositivos.
//              getTopologyBySede, getMetricsByDevice, getRecentMetrics.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional)

import { apiClient } from './client';
import type {
	SedeResponse,
	DeviceResponse,
	AlertResponse,
	MetricReading,
} from '$lib/generated/api-types';

export interface TopologyNode {
	id: string;
	label: string;
	type: 'sede' | 'device';
	data: SedeResponse | DeviceResponse;
}

export interface TopologyEdge {
	source: string;
	target: string;
	label?: string;
}

export interface TopologyResponse {
	nodes: TopologyNode[];
	edges: TopologyEdge[];
}

export async function getTopologyBySede(sedeId: string): Promise<TopologyResponse> {
	return apiClient.get<TopologyResponse>(`/api/v1/topology/sede/${sedeId}`);
}

export async function getMetricsByDevice(
	deviceId: string,
	interval?: string
): Promise<MetricReading[]> {
	const params = interval ? `?interval=${interval}` : '';
	return apiClient.get<MetricReading[]>(`/api/v1/metrics/device/${deviceId}${params}`);
}

export async function getRecentMetrics(limit?: number): Promise<MetricReading[]> {
	const params = limit ? `?limit=${limit}` : '';
	return apiClient.get<MetricReading[]>(`/api/v1/metrics/recent${params}`);
}
