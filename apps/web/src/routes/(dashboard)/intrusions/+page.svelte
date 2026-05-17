<!-- Ubicación: `apps/web/src/routes/(dashboard)/intrusions/+page.svelte` -->
<!-- Descripción: Página de eventos de intrusión detectados con tabla y acciones -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional), 0022 (Agentes) -->
<script lang="ts">
	import { ShieldAlert, CheckCircle, AlertTriangle, Ban } from 'lucide-svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { createQuery, useQueryClient } from '@tanstack/svelte-query';
	import { toast } from 'svelte-sonner';
	import { cn } from '$lib/utils';

	let statusFilter = $state<string | null>(null);

	const queryClient = useQueryClient();

	let intrusionsQuery = createQuery({
		queryKey: ['intrusions', statusFilter],
		queryFn: async () => {
			const params = new URLSearchParams();
			if (statusFilter) params.set('status', statusFilter);
			const response = await fetch(
				`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/intrusions?${params}`
			);
			if (!response.ok) throw new Error('Failed to fetch intrusions');
			return response.json();
		},
		staleTime: 30 * 1000,
		refetchInterval: 60 * 1000,
	});

	async function handleResolve(id: string) {
		try {
			const response = await fetch(
				`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/intrusions/${id}/resolve`,
				{ method: 'PUT' }
			);
			if (!response.ok) throw new Error('Failed to resolve intrusion');
			toast.success('Intrusión marcada como resuelta');
			queryClient.invalidateQueries({ queryKey: ['intrusions'] });
		} catch {
			toast.error('Error al resolver intrusión');
		}
	}

	async function handleFalsePositive(id: string) {
		try {
			const response = await fetch(
				`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/intrusions/${id}/false-positive`,
				{ method: 'PUT' }
			);
			if (!response.ok) throw new Error('Failed to mark as false positive');
			toast.info('Intrusión marcada como falso positivo');
			queryClient.invalidateQueries({ queryKey: ['intrusions'] });
		} catch {
			toast.error('Error al marcar intrusión');
		}
	}

	function getSeverityColor(severity: string) {
		switch (severity.toLowerCase()) {
			case 'critical':
				return 'bg-red-500/10 text-red-500 border-red-500/20';
			case 'high':
				return 'bg-orange-500/10 text-orange-500 border-orange-500/20';
			case 'medium':
				return 'bg-yellow-500/10 text-yellow-500 border-yellow-500/20';
			default:
				return 'bg-blue-500/10 text-blue-500 border-blue-500/20';
		}
	}

	function formatDate(dateStr: string) {
		return new Date(dateStr).toLocaleString('es-BO');
	}
</script>

<div class="space-y-6">
	<div>
		<h1 class="text-3xl font-bold tracking-tight">Intrusiones</h1>
		<p class="text-muted-foreground">Eventos de intrusión detectados por el sistema</p>
	</div>

	<div class="flex items-center gap-4">
		<label class="text-sm font-medium" for="status-filter">Estado:</label>
		<select
			id="status-filter"
			bind:value={statusFilter}
			class="flex h-10 rounded-md border border-input bg-background px-3 py-2 text-sm"
		>
			<option value={null}>Todos</option>
			<option value="detected">Detectado</option>
			<option value="investigating">Investigando</option>
			<option value="resolved">Resuelto</option>
			<option value="false_positive">Falso Positivo</option>
		</select>
	</div>

	{#if $intrusionsQuery.isLoading}
		<Card>
			<CardContent class="p-6">
				<div class="space-y-3">
					{#each Array(5) as _}
						<div class="h-20 animate-pulse rounded bg-muted" />
					{/each}
				</div>
			</CardContent>
		</Card>
	{:else if $intrusionsQuery.data?.length === 0}
		<Card>
			<CardContent class="flex flex-col items-center justify-center py-12">
				<ShieldAlert class="h-12 w-12 text-green-500 mb-4" />
				<p class="text-lg font-medium">Sin intrusiones</p>
				<p class="text-muted-foreground">No se han detectado intrusiones recently</p>
			</CardContent>
		</Card>
	{:else}
		<div class="space-y-4">
			{#each $intrusionsQuery.data ?? [] as intrusion (intrusion.id)}
				<Card>
					<CardContent class="p-4">
						<div class="flex items-start gap-4">
							<div class={cn('flex h-10 w-10 items-center justify-center rounded-full', getSeverityColor(intrusion.severity))}>
								<ShieldAlert class="h-5 w-5" />
							</div>

							<div class="flex-1 space-y-2">
								<div class="flex items-center gap-2">
									<h3 class="font-medium">{intrusion.intrusion_type}</h3>
									<Badge class={cn('text-xs', getSeverityColor(intrusion.severity))}>
										{intrusion.severity}
									</Badge>
									<Badge variant="outline" class="text-xs">
										{intrusion.status}
									</Badge>
								</div>

								<div class="grid grid-cols-2 gap-x-8 gap-y-1 text-sm text-muted-foreground">
									<div>
										<span class="font-medium">IP Origen:</span> {intrusion.source_ip}
									</div>
									<div>
										<span class="font-medium">MAC:</span> {intrusion.mac_address}
									</div>
									{#if intrusion.device_id}
										<div>
											<span class="font-medium">Dispositivo:</span> {intrusion.device_id}
										</div>
									{/if}
									<div>
										<span class="font-medium">Detectado:</span> {formatDate(intrusion.detected_at)}
									</div>
								</div>
							</div>

							<div class="flex gap-2">
								{#if intrusion.status !== 'resolved' && intrusion.status !== 'false_positive'}
									<Button
										variant="outline"
										size="sm"
										onclick={() => handleResolve(intrusion.id)}
									>
										<CheckCircle class="mr-1 h-4 w-4" />
										Resolver
									</Button>
									<Button
										variant="outline"
										size="sm"
										onclick={() => handleFalsePositive(intrusion.id)}
									>
										<Ban class="mr-1 h-4 w-4" />
										Falso Positivo
									</Button>
								{/if}
							</div>
						</div>
					</CardContent>
				</Card>
			{/each}
		</div>
	{/if}
</div>
