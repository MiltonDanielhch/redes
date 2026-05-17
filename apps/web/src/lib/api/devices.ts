// Ubicación: `apps/web/src/lib/api/devices.ts`
//
// Descripción: API module para gestión de dispositivos de red.
//              CRUD completo: listDevices, getDevice, createDevice, updateDevice, deleteDevice.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional)

import { apiClient } from './client';
import type { DeviceResponse, CreateDeviceRequest, UpdateDeviceRequest } from '$lib/generated/api-types';

export async function listDevices(): Promise<DeviceResponse[]> {
	return apiClient.get<DeviceResponse[]>('/api/v1/devices');
}

export async function getDevice(id: string): Promise<DeviceResponse> {
	return apiClient.get<DeviceResponse>(`/api/v1/devices/${id}`);
}

export async function createDevice(data: CreateDeviceRequest): Promise<DeviceResponse> {
	return apiClient.post<DeviceResponse>('/api/v1/devices', data);
}

export async function updateDevice(id: string, data: UpdateDeviceRequest): Promise<DeviceResponse> {
	return apiClient.put<DeviceResponse>(`/api/v1/devices/${id}`, data);
}

export async function deleteDevice(id: string): Promise<void> {
	return apiClient.delete<void>(`/api/v1/devices/${id}`);
}
