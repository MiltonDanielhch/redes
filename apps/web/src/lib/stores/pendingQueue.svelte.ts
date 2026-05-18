// Ubicación: `apps/web/src/lib/stores/pendingQueue.svelte.ts`
//
// Descripción: Queue de acciones pendientes para sincronizar cuando hay conexión.
//              Las acciones se ejecutan en orden FIFO cuando se recupera la conexión.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0021 (Local-First)

import { network } from './network.svelte';

interface PendingAction {
	id: string;
	type: 'acknowledge_alert' | 'resolve_alert' | 'update_device' | 'create_device' | string;
	payload: unknown;
	timestamp: string;
	retries: number;
}

const MAX_RETRIES = 3;
const STORAGE_KEY = 'pending_actions';

function loadFromStorage(): PendingAction[] {
	if (typeof window === 'undefined') return [];
	try {
		const stored = localStorage.getItem(STORAGE_KEY);
		return stored ? JSON.parse(stored) : [];
	} catch {
		return [];
	}
}

function saveToStorage(actions: PendingAction[]) {
	if (typeof window === 'undefined') return;
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(actions));
	} catch (e) {
		console.error('Failed to save pending actions:', e);
	}
}

function createPendingQueueStore() {
	let queue = $state<PendingAction[]>(loadFromStorage());
	let isSyncing = $state(false);

	async function executeAction(action: PendingAction): Promise<boolean> {
		try {
			let endpoint = '';
			let method = 'POST';

			switch (action.type) {
				case 'acknowledge_alert':
					endpoint = `/api/v1/alerts/${(action.payload as { id: string }).id}/acknowledge`;
					break;
				case 'resolve_alert':
					endpoint = `/api/v1/alerts/${(action.payload as { id: string }).id}/resolve`;
					break;
				case 'update_device':
					endpoint = `/api/v1/devices/${(action.payload as { id: string }).id}`;
					method = 'PUT';
					break;
				case 'create_device':
					endpoint = '/api/v1/devices';
					break;
				default:
					console.warn('Unknown action type:', action.type);
					return false;
			}

			const response = await fetch(
				`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}${endpoint}`,
				{
					method,
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify(action.payload),
				}
			);

			return response.ok;
		} catch {
			return false;
		}
	}

	async function sync() {
		if (isSyncing || !network.isOnline || queue.length === 0) return;

		isSyncing = true;

		while (queue.length > 0 && network.isOnline) {
			const action = queue[0];

			const success = await executeAction(action);

			if (success) {
				queue = queue.slice(1);
				saveToStorage(queue);
			} else {
				action.retries++;

				if (action.retries >= MAX_RETRIES) {
					console.error('Action failed after max retries:', action);
					queue = queue.slice(1);
					saveToStorage(queue);
				} else {
					await new Promise((resolve) => setTimeout(resolve, 1000 * action.retries));
				}
			}
		}

		isSyncing = false;
	}

	function addAction(type: PendingAction['type'], payload: unknown) {
		const action: PendingAction = {
			id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
			type,
			payload,
			timestamp: new Date().toISOString(),
			retries: 0,
		};

		queue = [...queue, action];
		saveToStorage(queue);

		if (network.isOnline) {
			sync();
		}
	}

	function removeAction(id: string) {
		queue = queue.filter((a) => a.id !== id);
		saveToStorage(queue);
	}

	function clearQueue() {
		queue = [];
		saveToStorage(queue);
	}

	function getQueue() {
		return queue;
	}

	return {
		get queue() { return queue; },
		get isSyncing() { return isSyncing; },
		addAction,
		removeAction,
		clearQueue,
		getQueue,
		sync,
	};
}

export const pendingQueue = createPendingQueueStore();
