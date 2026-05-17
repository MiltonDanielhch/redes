<!-- Ubicación: `apps/web/src/lib/components/dashboard/StatsOverview.svelte` -->
<!-- Descripción: Overview de estadísticas globales con createQuery y staleTime de 1 minuto -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional), 0021 (Offline) -->
<script lang="ts">
	import { Building2, Monitor, AlertTriangle, TrendingUp } from 'lucide-svelte';
	import KpiCard from './KpiCard.svelte';
	import { createQuery } from '@tanstack/svelte-query';

	interface StatsData {
		totalSedes: number;
		activeDevices: number;
		pendingAlerts: number;
		bandwidthGbps: number;
	}

	let queryOptions = {
		queryKey: ['stats', 'overview'],
		queryFn: async (): Promise<StatsData> => {
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/stats/overview`);
			if (!response.ok) throw new Error('Failed to fetch stats');
			return response.json();
		},
		staleTime: 60 * 1000,
		retry: 1,
	};

	let query = createQuery(queryOptions);

	let loading = $derived($query.isLoading || $query.isFetching);
	let stats = $derived($query.data);
</script>

<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
	<KpiCard
		title="Sedes Activas"
		value={stats?.totalSedes ?? 0}
		icon={Building2}
		description="+2 desde el mes pasado"
		trend={{ value: 5, direction: 'up' }}
		href="/dashboard/sedes"
		{loading}
	/>

	<KpiCard
		title="Dispositivos Online"
		value={stats?.activeDevices ?? 0}
		icon={Monitor}
		description="98.2% disponibilidad"
		trend={{ value: 2, direction: 'up' }}
		href="/dashboard/devices"
		{loading}
	/>

	<KpiCard
		title="Alertas Pendientes"
		value={stats?.pendingAlerts ?? 0}
		icon={AlertTriangle}
		description="5 críticas"
		trend={{ value: 12, direction: 'down' }}
		href="/dashboard/alerts"
		{loading}
	/>

	<KpiCard
		title="Ancho de Banda"
		value={`${stats?.bandwidthGbps ?? 0} Gbps`}
		icon={TrendingUp}
		description="+12% promedio"
		trend={{ value: 12, direction: 'up' }}
		href="/dashboard/metrics"
		{loading}
	/>
</div>
