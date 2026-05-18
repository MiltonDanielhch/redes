<!-- Ubicación: `apps/web/src/lib/components/network/OfflineBanner.svelte` -->
<!-- Descripción: Banner que muestra estado offline con información de sincronización pendiente -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0021 (Local-First) -->
<script lang="ts">
	import { WifiOff, RefreshCw, CheckCircle } from 'lucide-svelte';
	import { network } from '$lib/stores/network.svelte';
	import { pendingQueue } from '$lib/stores/pendingQueue.svelte';

	let pendingCount = $derived(pendingQueue.queue.length);
</script>

{#if !network.isOnline}
	<div class="fixed bottom-0 left-0 right-0 z-50 bg-yellow-500/90 text-yellow-950 px-4 py-2">
		<div class="flex items-center justify-between gap-4">
			<div class="flex items-center gap-2">
				<WifiOff class="h-5 w-5" />
				<span class="text-sm font-medium">
					Modo offline — los datos pueden estar desactualizados
				</span>
			</div>

			{#if pendingCount > 0}
				<div class="flex items-center gap-2 text-sm">
					<RefreshCw class="h-4 w-4 animate-spin" />
					<span>{pendingCount} acción(es) pendiente(s)</span>
				</div>
			{/if}
		</div>
	</div>
{:else if network.wasOffline && pendingCount > 0}
	<div class="fixed bottom-0 left-0 right-0 z-50 bg-green-500/90 text-green-950 px-4 py-2">
		<div class="flex items-center justify-between gap-4">
			<div class="flex items-center gap-2">
				<CheckCircle class="h-5 w-5" />
				<span class="text-sm font-medium">
					Conexión restaurada — sincronizando {pendingCount} acción(es)...
				</span>
			</div>

			{#if pendingQueue.isSyncing}
				<RefreshCw class="h-4 w-4 animate-spin" />
			{/if}
		</div>
	</div>
{/if}
