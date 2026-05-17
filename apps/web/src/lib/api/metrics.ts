// Ubicación: `apps/web/src/lib/api/metrics.ts`
//
// Descripción: API module para obtener métricas de dispositivos y ancho de banda.
//              getRecentMetrics, getMetricsByDevice, getAggregatedMetrics.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional)

import { apiClient } from './client';
import type { MetricReading } from '$lib/generated/api-types';

export interface AggregatedMetrics {
	cpu_avg: number;
	memory_avg: number;
	bandwidth_rx_mbps: number;
	bandwidth_tx_mbps: number;
	packet_loss_pct: number;
	latency_ms: number;
}

export interface MetricsFilter {
	device_id?: string;
	sede_id?: string;
	from?: string;
	to?: string;
	interval?: '1m' | '5m' | '15m' | '1h' | '1d';
}

export async function getRecentMetrics(limit = 100): Promise<MetricReading[]> {
	const params = new URLSearchParams({ limit: limit.toString() });
	return apiClient.get<MetricReading[]>(`/api/v1/metrics/recent?${params}`);
}

export async function getMetricsByDevice(
	deviceId: string,
	filter?: MetricsFilter
): Promise<MetricReading[]> {
	const params = new URLSearchParams();
	if (filter?.from) params.set('from', filter.from);
	if (filter?.to) params.set('to', filter.to);
	if (filter?.interval) params.set('interval', filter.interval);
	return apiClient.get<MetricReading[]>(`/api/v1/metrics/device/${deviceId}?${params}`);
}

export async function getAggregatedMetrics(
	filter?: MetricsFilter
): Promise<AggregatedMetrics> {
	const params = new URLSearchParams();
	if (filter?.sede_id) params.set('sede_id', filter.sede_id);
	if (filter?.from) params.set('from', filter.from);
	if (filter?.to) params.set('to', filter.to);
	return apiClient.get<AggregatedMetrics>(`/api/v1/metrics/aggregated?${params}`);
}

export async function getMetricsBySede(sedeId: string): Promise<MetricReading[]> {
	return apiClient.get<MetricReading[]>(`/api/v1/metrics/sede/${sedeId}`);
}
