<!-- Ubicación: `apps/web/src/lib/components/alerts/AlertTable.svelte` -->
<!-- Descripción: Tabla de alertas con filtros, paginación y acciones -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { AlertTriangle, Clock, CheckCircle, XCircle } from 'lucide-svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { cn } from '$lib/utils';
	import type { AlertResponse } from '$lib/generated/api-types';

	interface Props {
		alerts?: AlertResponse[];
		loading?: boolean;
		onAcknowledge?: (alertId: string) => void;
		onResolve?: (alertId: string) => void;
		onViewDetails?: (alertId: string) => void;
	}

	let {
		alerts = [],
		loading = false,
		onAcknowledge,
		onResolve,
		onViewDetails
	}: Props = $props();

	let searchQuery = $state('');
	let statusFilter = $state<string | undefined>(undefined);
	let severityFilter = $state<string | undefined>(undefined);

	let filteredAlerts = $derived(
		alerts.filter((alert) => {
			const matchesSearch =
				!searchQuery ||
				alert.message.toLowerCase().includes(searchQuery.toLowerCase()) ||
				alert.deviceId?.toLowerCase().includes(searchQuery.toLowerCase());

			const matchesStatus = !statusFilter || alert.status === statusFilter;
			const matchesSeverity = !severityFilter || alert.severity === severityFilter;

			return matchesSearch && matchesStatus && matchesSeverity;
		})
	);

	function getSeverityColor(severity: string) {
		switch (severity.toLowerCase()) {
			case 'critical':
				return 'bg-red-500/10 text-red-500 border-red-500/20';
			case 'high':
				return 'bg-orange-500/10 text-orange-500 border-orange-500/20';
			case 'medium':
				return 'bg-yellow-500/10 text-yellow-500 border-yellow-500/20';
			case 'low':
				return 'bg-blue-500/10 text-blue-500 border-blue-500/20';
			default:
				return 'bg-gray-500/10 text-gray-500 border-gray-500/20';
		}
	}

	function getStatusColor(status: string) {
		switch (status) {
			case 'active':
				return 'bg-red-500';
			case 'acknowledged':
				return 'bg-yellow-500';
			case 'resolved':
				return 'bg-green-500';
			default:
				return 'bg-gray-500';
		}
	}

	function formatTimeAgo(dateStr: string) {
		const date = new Date(dateStr);
		const now = new Date();
		const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

		if (seconds < 60) return 'Hace un momento';
		if (seconds < 3600) return `Hace ${Math.floor(seconds / 60)} min`;
		if (seconds < 86400) return `Hace ${Math.floor(seconds / 3600)}h`;
		return `Hace ${Math.floor(seconds / 86400)}d`;
	}
</script>

<div class="space-y-4">
	<div class="flex flex-wrap items-center gap-4">
		<Input
			placeholder="Buscar alertas..."
			bind:value={searchQuery}
			class="max-w-sm"
		/>

		<select
			bind:value={statusFilter}
			class="flex h-10 rounded-md border border-input bg-background px-3 py-2 text-sm"
		>
			<option value={undefined}>Todos los estados</option>
			<option value="active">Activas</option>
			<option value="acknowledged">Reconocidas</option>
			<option value="resolved">Resueltas</option>
		</select>

		<select
			bind:value={severityFilter}
			class="flex h-10 rounded-md border border-input bg-background px-3 py-2 text-sm"
		>
			<option value={undefined}>Todas</option>
			<option value="Critical">Crítica</option>
			<option value="High">Alta</option>
			<option value="Medium">Media</option>
			<option value="Low">Baja</option>
		</select>
	</div>

	{#if loading}
		<Card>
			<CardContent class="p-6">
				<div class="space-y-3">
					{#each Array(5) as _}
						<div class="h-16 animate-pulse rounded bg-muted" />
					{/each}
				</div>
			</CardContent>
		</Card>
	{:else if filteredAlerts.length === 0}
		<Card>
			<CardContent class="flex flex-col items-center justify-center py-12">
				<CheckCircle class="h-12 w-12 text-green-500 mb-4" />
				<p class="text-lg font-medium">Sin alertas</p>
				<p class="text-muted-foreground">No hay alertas que coincidan con los filtros</p>
			</CardContent>
		</Card>
	{:else}
		<Card>
			<CardContent class="p-0">
				<div class="divide-y">
					{#each filteredAlerts as alert (alert.id)}
						<div class="flex items-center gap-4 p-4 hover:bg-muted/50">
							<div class={cn('flex h-10 w-10 items-center justify-center rounded-full', getSeverityColor(alert.severity))}>
								<AlertTriangle class="h-5 w-5" />
							</div>

							<div class="flex-1 space-y-1">
								<div class="flex items-center gap-2">
									<p class="font-medium">{alert.message}</p>
									<Badge class={cn('text-xs', getSeverityColor(alert.severity))}>
										{alert.severity}
									</Badge>
								</div>
								<div class="flex items-center gap-4 text-xs text-muted-foreground">
									<span class="flex items-center gap-1">
										<Clock class="h-3 w-3" />
										{formatTimeAgo(alert.createdAt)}
									</span>
									{#if alert.deviceId}
										<span>Dispositivo: {alert.deviceId}</span>
									{/if}
								</div>
							</div>

							<div class="flex items-center gap-2">
								{#if alert.status === 'active'}
									<Button
										variant="outline"
										size="sm"
										onclick={() => onAcknowledge?.(alert.id)}
									>
										<CheckCircle class="mr-1 h-4 w-4" />
										Reconocer
									</Button>
								{/if}
								{#if alert.status !== 'resolved'}
									<Button
										variant="outline"
										size="sm"
										onclick={() => onResolve?.(alert.id)}
									>
										<XCircle class="mr-1 h-4 w-4" />
										Resolver
									</Button>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			</CardContent>
		</Card>
	{/if}
</div>
