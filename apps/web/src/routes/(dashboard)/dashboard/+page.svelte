<!-- Ubicación: `apps/web/src/routes/(dashboard)/dashboard/+page.svelte` -->
<!-- Descripción: Dashboard principal con resumen de métricas, alertas y estado de red -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import StatsOverview from '$lib/components/dashboard/StatsOverview.svelte';
	import AlertSummary from '$lib/components/dashboard/AlertSummary.svelte';
	import DeviceStatusChart from '$lib/components/dashboard/DeviceStatusChart.svelte';
	import NetworkHealth from '$lib/components/dashboard/NetworkHealth.svelte';
	import AgentStatusWidget from '$lib/components/dashboard/AgentStatusWidget.svelte';
	import { createQuery } from '@tanstack/svelte-query';
	import type { AlertResponse } from '$lib/generated/api-types';

	let alertsQuery = createQuery({
		queryKey: ['alerts', 'recent'],
		queryFn: async (): Promise<AlertResponse[]> => {
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/alerts?limit=5&severity=Critical,High`);
			if (!response.ok) throw new Error('Failed to fetch alerts');
			const data = await response.json();
			return data.alerts || [];
		},
		staleTime: 30 * 1000,
		refetchInterval: 30 * 1000,
	});

	let deviceStatusQuery = createQuery({
		queryKey: ['devices', 'status-count'],
		queryFn: async () => {
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/devices/status-count`);
			if (!response.ok) throw new Error('Failed to fetch device status');
			return response.json();
		},
		staleTime: 60 * 1000,
	});

	let networkHealthQuery = createQuery({
		queryKey: ['network', 'health'],
		queryFn: async () => {
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/network/health`);
			if (!response.ok) throw new Error('Failed to fetch network health');
			return response.json();
		},
		staleTime: 60 * 1000,
	});

	let agentsQuery = createQuery({
		queryKey: ['agents', 'recent'],
		queryFn: async () => {
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/agents?limit=5`);
			if (!response.ok) throw new Error('Failed to fetch agents');
			return response.json();
		},
		staleTime: 60 * 1000,
	});

	function handleAlertClick(alertId: string) {
		window.location.href = `/dashboard/alerts?id=${alertId}`;
	}

	function handleAgentClick(agentId: string) {
		window.location.href = `/dashboard/agents?id=${agentId}`;
	}
</script>

<div class="space-y-6">
	<div>
		<h1 class="text-3xl font-bold tracking-tight">Dashboard</h1>
		<p class="text-muted-foreground">Resumen del monitoreo de infraestructura regional</p>
	</div>

	<StatsOverview />

	<div class="grid gap-4 md:grid-cols-2">
		<AlertSummary
			alerts={$alertsQuery.data}
			loading={$alertsQuery.isLoading}
			onAlertClick={handleAlertClick}
		/>

		<DeviceStatusChart
			data={$deviceStatusQuery.data}
			loading={$deviceStatusQuery.isLoading}
		/>
	</div>

	<div class="grid gap-4 md:grid-cols-2">
		<NetworkHealth
			latencyMs={$networkHealthQuery.data?.latency_ms}
			packetLossPct={$networkHealthQuery.data?.packet_loss_pct}
			uptimePct={$networkHealthQuery.data?.uptime_pct}
			trend={$networkHealthQuery.data?.trend}
			loading={$networkHealthQuery.isLoading}
		/>

		<AgentStatusWidget
			agents={$agentsQuery.data}
			loading={$agentsQuery.isLoading}
			onAgentClick={handleAgentClick}
		/>
	</div>
</div>
