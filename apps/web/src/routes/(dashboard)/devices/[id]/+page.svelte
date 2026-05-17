<!-- Ubicación: `apps/web/src/routes/(dashboard)/devices/[id]/+page.svelte` -->
<!-- Descripción: Vista detallada de dispositivo con métricas, alertas y topología -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { ArrowLeft, Monitor, MapPin, Clock, Activity, AlertTriangle } from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Separator } from '$lib/components/ui/separator';
	import { createQuery } from '@tanstack/svelte-query';
	import { page } from '$app/stores';

	let deviceId = $derived($page.params.id);

	let deviceQuery = createQuery({
		queryKey: ['devices', deviceId],
		queryFn: async () => {
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/devices/${deviceId}`);
			if (!response.ok) throw new Error('Failed to fetch device');
			return response.json();
		},
		enabled: !!deviceId,
	});

	let alertsQuery = createQuery({
		queryKey: ['alerts', 'device', deviceId],
		queryFn: async () => {
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/alerts?device_id=${deviceId}`);
			if (!response.ok) throw new Error('Failed to fetch alerts');
			return response.json();
		},
		enabled: !!deviceId,
	});

	function getStatusColor(status: string) {
		switch (status) {
			case 'active':
				return 'bg-green-500/10 text-green-500 border-green-500/20';
			case 'offline':
				return 'bg-red-500/10 text-red-500 border-red-500/20';
			case 'maintenance':
				return 'bg-blue-500/10 text-blue-500 border-blue-500/20';
			default:
				return 'bg-gray-500/10 text-gray-500 border-gray-500/20';
		}
	}
</script>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<Button variant="ghost" size="icon" href="/dashboard/devices">
			<ArrowLeft class="h-4 w-4" />
		</Button>
		<div>
			<h1 class="text-3xl font-bold tracking-tight">
				{$deviceQuery.data?.hostname ?? 'Cargando...'}
			</h1>
			<p class="text-muted-foreground">Detalles del dispositivo</p>
		</div>
	</div>

	{#if $deviceQuery.isLoading}
		<div class="grid gap-4 md:grid-cols-2">
			<Card><CardContent class="p-6"><div class="h-32 animate-pulse rounded bg-muted" /></CardContent></Card>
			<Card><CardContent class="p-6"><div class="h-32 animate-pulse rounded bg-muted" /></CardContent></Card>
		</div>
	{:else if $deviceQuery.data}
		<div class="grid gap-4 md:grid-cols-2">
			<Card>
				<CardHeader>
					<CardTitle class="flex items-center gap-2">
						<Monitor class="h-5 w-5" />
						Información General
					</CardTitle>
				</CardHeader>
				<CardContent class="space-y-4">
					<div class="grid grid-cols-2 gap-4">
						<div>
							<p class="text-sm text-muted-foreground">Hostname</p>
							<p class="font-medium">{$deviceQuery.data.hostname}</p>
						</div>
						<div>
							<p class="text-sm text-muted-foreground">Tipo</p>
							<p class="font-medium capitalize">{$deviceQuery.data.device_type}</p>
						</div>
						<div>
							<p class="text-sm text-muted-foreground">Dirección IP</p>
							<p class="font-medium">{$deviceQuery.data.ip_address}</p>
						</div>
						<div>
							<p class="text-sm text-muted-foreground">Dirección MAC</p>
							<p class="font-medium">{$deviceQuery.data.mac_address ?? 'N/A'}</p>
						</div>
					</div>
					<Separator />
					<div class="grid grid-cols-2 gap-4">
						<div>
							<p class="text-sm text-muted-foreground">Estado</p>
							<Badge class={getStatusColor($deviceQuery.data.status)}>
								{$deviceQuery.data.status}
							</Badge>
						</div>
						<div>
							<p class="text-sm text-muted-foreground">Última conexión</p>
							<p class="font-medium flex items-center gap-1">
								<Clock class="h-4 w-4" />
								{$deviceQuery.data.last_seen_at
									? new Date($deviceQuery.data.last_seen_at).toLocaleString('es-BO')
									: 'Nunca'}
							</p>
						</div>
					</div>
				</CardContent>
			</Card>

			<Card>
				<CardHeader>
					<CardTitle class="flex items-center gap-2">
						<Activity class="h-5 w-5" />
						Métricas en Tiempo Real
					</CardTitle>
				</CardHeader>
				<CardContent>
					<p class="text-muted-foreground">Conectando con streaming de métricas...</p>
					<div class="mt-4 grid grid-cols-3 gap-4">
						<div class="text-center">
							<p class="text-2xl font-bold text-green-500">45%</p>
							<p class="text-xs text-muted-foreground">CPU</p>
						</div>
						<div class="text-center">
							<p class="text-2xl font-bold text-blue-500">62%</p>
							<p class="text-xs text-muted-foreground">Memoria</p>
						</div>
						<div class="text-center">
							<p class="text-2xl font-bold text-purple-500">1.2 Gbps</p>
							<p class="text-xs text-muted-foreground">Ancho de banda</p>
						</div>
					</div>
				</CardContent>
			</Card>
		</div>

		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<AlertTriangle class="h-5 w-5" />
					Alertas Recientes
				</CardTitle>
			</CardHeader>
			<CardContent>
				{#if $alertsQuery.data?.alerts?.length > 0}
					<div class="space-y-2">
						{#each $alertsQuery.data.alerts.slice(0, 5) as alert}
							<div class="flex items-center justify-between rounded-lg border p-3">
								<div class="flex items-center gap-3">
									<AlertTriangle class="h-4 w-4 text-yellow-500" />
									<div>
										<p class="text-sm font-medium">{alert.message}</p>
										<p class="text-xs text-muted-foreground">
											{new Date(alert.createdAt).toLocaleString('es-BO')}
										</p>
									</div>
								</div>
								<Badge variant="outline">{alert.severity}</Badge>
							</div>
						{/each}
					</div>
				{:else}
					<p class="text-center text-muted-foreground py-4">Sin alertas para este dispositivo</p>
				{/if}
			</CardContent>
		</Card>
	{/if}
</div>
