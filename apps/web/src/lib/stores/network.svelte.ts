// Ubicación: `apps/web/src/lib/stores/network.svelte.ts`
//
// Descripción: Store de conectividad de red con detection de online/offline.
//              Usa navigator.onLine y event listeners para cambios.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0021 (Local-First)

import { onMount, onDestroy } from 'svelte';

interface NetworkState {
	isOnline: boolean;
	wasOffline: boolean;
	lastOnlineAt: string | null;
}

function createNetworkStore() {
	let isOnline = $state(true);
	let wasOffline = $state(false);
	let lastOnlineAt = $state<string | null>(null);

	function handleOnline() {
		isOnline = true;
		wasOffline = true;
		lastOnlineAt = new Date().toISOString();
	}

	function handleOffline() {
		isOnline = false;
	}

	onMount(() => {
		if (typeof window !== 'undefined') {
			isOnline = navigator.onLine;

			window.addEventListener('online', handleOnline);
			window.addEventListener('offline', handleOffline);
		}
	});

	onDestroy(() => {
		if (typeof window !== 'undefined') {
			window.removeEventListener('online', handleOnline);
			window.removeEventListener('offline', handleOffline);
		}
	});

	function resetWasOffline() {
		wasOffline = false;
	}

	return {
		get isOnline() { return isOnline; },
		get wasOffline() { return wasOffline; },
		get lastOnlineAt() { return lastOnlineAt; },
		resetWasOffline,
	};
}

export const network = createNetworkStore();
