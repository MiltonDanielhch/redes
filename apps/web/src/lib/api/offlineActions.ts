// Ubicación: `apps/web/src/lib/api/offlineActions.ts`
//
// Descripción: Helpers para acciones que funcionan offline-first.
//              Usan pendingQueue cuando hay conexión lenta o sin conexión.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0021 (Local-First)

import { pendingQueue } from '$lib/stores/pendingQueue.svelte';
import { network } from '$lib/stores/network.svelte';

export interface ActionResult {
	success: boolean;
	pending?: boolean;
}

export async function acknowledgeAlertOffline(alertId: string): Promise<ActionResult> {
	if (!network.isOnline) {
		pendingQueue.addAction('acknowledge_alert', { id: alertId });
		return { success: true, pending: true };
	}

	try {
		const response = await fetch(
			`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/alerts/${alertId}/acknowledge`,
			{ method: 'PUT' }
		);

		if (!response.ok) {
			pendingQueue.addAction('acknowledge_alert', { id: alertId });
			return { success: true, pending: true };
		}

		return { success: true };
	} catch {
		pendingQueue.addAction('acknowledge_alert', { id: alertId });
		return { success: true, pending: true };
	}
}

export async function resolveAlertOffline(alertId: string): Promise<ActionResult> {
	if (!network.isOnline) {
		pendingQueue.addAction('resolve_alert', { id: alertId });
		return { success: true, pending: true };
	}

	try {
		const response = await fetch(
			`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/alerts/${alertId}/resolve`,
			{ method: 'PUT' }
		);

		if (!response.ok) {
			pendingQueue.addAction('resolve_alert', { id: alertId });
			return { success: true, pending: true };
		}

		return { success: true };
	} catch {
		pendingQueue.addAction('resolve_alert', { id: alertId });
		return { success: true, pending: true };
	}
}

export async function updateDeviceOffline(
	deviceId: string,
	data: Record<string, unknown>
): Promise<ActionResult> {
	const payload = { id: deviceId, ...data };

	if (!network.isOnline) {
		pendingQueue.addAction('update_device', payload);
		return { success: true, pending: true };
	}

	try {
		const response = await fetch(
			`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/devices/${deviceId}`,
			{
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(data),
			}
		);

		if (!response.ok) {
			pendingQueue.addAction('update_device', payload);
			return { success: true, pending: true };
		}

		return { success: true };
	} catch {
		pendingQueue.addAction('update_device', payload);
		return { success: true, pending: true };
	}
}
