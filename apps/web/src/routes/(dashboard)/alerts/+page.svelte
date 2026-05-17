<!-- Ubicación: `apps/web/src/routes/(dashboard)/alerts/+page.svelte` -->
<!-- Descripción: Página de gestión de alertas con tabla, filtros y acciones -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import AlertTable from '$lib/components/alerts/AlertTable.svelte';
	import AlertFilters from '$lib/components/alerts/AlertFilters.svelte';
	import { createQuery, useQueryClient } from '@tanstack/svelte-query';
	import { toast } from 'svelte-sonner';
	import { page } from '$app/stores';

	let severityFilter = $state<string[]>([]);
	let statusFilter = $state<string[]>([]);

	const queryClient = useQueryClient();

	let alertsQuery = createQuery({
		queryKey: ['alerts', severityFilter, statusFilter],
		queryFn: async () => {
			const params = new URLSearchParams();
			if (severityFilter.length > 0) {
				params.set('severity', severityFilter.join(','));
			}
			if (statusFilter.length > 0) {
				params.set('status', statusFilter.join(','));
			}
			const response = await fetch(
				`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/alerts?${params}`
			);
			if (!response.ok) throw new Error('Failed to fetch alerts');
			const data = await response.json();
			return data.alerts || [];
		},
		staleTime: 30 * 1000,
		refetchInterval: 30 * 1000,
	});

	async function handleAcknowledge(alertId: string) {
		try {
			const response = await fetch(
				`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/alerts/${alertId}/acknowledge`,
				{ method: 'PUT' }
			);
			if (!response.ok) throw new Error('Failed to acknowledge alert');
			toast.success('Alerta reconocida');
			queryClient.invalidateQueries({ queryKey: ['alerts'] });
		} catch {
			toast.error('Error al reconocer alerta');
		}
	}

	async function handleResolve(alertId: string) {
		try {
			const response = await fetch(
				`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/alerts/${alertId}/resolve`,
				{ method: 'PUT' }
			);
			if (!response.ok) throw new Error('Failed to resolve alert');
			toast.success('Alerta resuelta');
			queryClient.invalidateQueries({ queryKey: ['alerts'] });
		} catch {
			toast.error('Error al resolver alerta');
		}
	}

	function handleViewDetails(alertId: string) {
		window.location.href = `/dashboard/alerts/${alertId}`;
	}

	function handleReset() {
		severityFilter = [];
		statusFilter = [];
	}
</script>

<div class="space-y-6">
	<div>
		<h1 class="text-3xl font-bold tracking-tight">Alertas</h1>
		<p class="text-muted-foreground">Monitoreo y gestión de alertas del sistema</p>
	</div>

	<div class="grid gap-6 lg:grid-cols-4">
		<div class="lg:col-span-1">
			<AlertFilters
				selectedSeverity={severityFilter}
				selectedStatuses={statusFilter}
				onSeverityChange={(s) => (severityFilter = s)}
				onStatusChange={(s) => (statusFilter = s)}
				onReset={handleReset}
			/>
		</div>

		<div class="lg:col-span-3">
			<AlertTable
				alerts={$alertsQuery.data}
				loading={$alertsQuery.isLoading}
				onAcknowledge={handleAcknowledge}
				onResolve={handleResolve}
				onViewDetails={handleViewDetails}
			/>
		</div>
	</div>
</div>
