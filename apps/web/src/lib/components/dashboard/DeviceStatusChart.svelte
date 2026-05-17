<!-- Ubicación: `apps/web/src/lib/components/dashboard/DeviceStatusChart.svelte` -->
<!-- Descripción: Gráfico donut mostrando estado de dispositivos (online, offline, maintenance) -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Skeleton } from '$lib/components/ui/skeleton';

	interface DeviceStatusCount {
		online: number;
		offline: number;
		maintenance: number;
	}

	interface Props {
		data?: DeviceStatusCount;
		loading?: boolean;
	}

	let { data, loading = false }: Props = $props();

	const colors = {
		online: '#22c55e',
		offline: '#ef4444',
		maintenance: '#3b82f6'
	};

	let total = $derived(data ? data.online + data.offline + data.maintenance : 0);

	let onlinePercent = $derived(data ? Math.round((data.online / total) * 100) : 0);
	let offlinePercent = $derived(data ? Math.round((data.offline / total) * 100) : 0);
	let maintenancePercent = $derived(data ? Math.round((data.maintenance / total) * 100) : 0);
</script>

<Card>
	<CardHeader>
		<CardTitle>Estado de Dispositivos</CardTitle>
	</CardHeader>
	<CardContent>
		{#if loading}
			<div class="flex items-center justify-center">
				<Skeleton class="h-48 w-48 rounded-full" />
			</div>
		{:else if data}
			<div class="flex flex-col items-center gap-4">
				<div class="relative h-48 w-48">
					<svg class="h-full w-full -rotate-90" viewBox="0 0 100 100">
						<circle
							cx="50"
							cy="50"
							r="40"
							fill="none"
							stroke={colors.offline}
							stroke-width="12"
							stroke-dasharray={`${(data.offline / total) * 251.2} 251.2`}
							stroke-dashoffset="0"
						/>
						<circle
							cx="50"
							cy="50"
							r="40"
							fill="none"
							stroke={colors.maintenance}
							stroke-width="12"
							stroke-dasharray={`${(data.maintenance / total) * 251.2} 251.2`}
							stroke-dashoffset={`-${(data.offline / total) * 251.2}`}
						/>
						<circle
							cx="50"
							cy="50"
							r="40"
							fill="none"
							stroke={colors.online}
							stroke-width="12"
							stroke-dasharray={`${(data.online / total) * 251.2} 251.2`}
							stroke-dashoffset={`-${(data.offline / total + data.maintenance / total) * 251.2}`}
						/>
					</svg>
					<div class="absolute inset-0 flex flex-col items-center justify-center">
						<span class="text-3xl font-bold">{total}</span>
						<span class="text-xs text-muted-foreground">dispositivos</span>
					</div>
				</div>

				<div class="flex flex-wrap justify-center gap-4">
					<div class="flex items-center gap-2">
						<div class="h-3 w-3 rounded-full bg-green-500" />
						<span class="text-sm">Online {onlinePercent}%</span>
					</div>
					<div class="flex items-center gap-2">
						<div class="h-3 w-3 rounded-full bg-red-500" />
						<span class="text-sm">Offline {offlinePercent}%</span>
					</div>
					<div class="flex items-center gap-2">
						<div class="h-3 w-3 rounded-full bg-blue-500" />
						<span class="text-sm">Maintenance {maintenancePercent}%</span>
					</div>
				</div>
			</div>
		{:else}
			<p class="text-center text-muted-foreground">Sin datos disponibles</p>
		{/if}
	</CardContent>
</Card>
