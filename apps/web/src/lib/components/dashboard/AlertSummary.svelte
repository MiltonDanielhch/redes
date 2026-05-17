<!-- Ubicación: `apps/web/src/lib/components/dashboard/AlertSummary.svelte` -->
<!-- Descripción: Muestra las últimas 5 alertas críticas/high con actualización periódica -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { AlertTriangle, Clock, CheckCircle } from 'lucide-svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { cn } from '$lib/utils';
	import type { AlertResponse } from '$lib/generated/api-types';

	interface Props {
		alerts?: AlertResponse[];
		loading?: boolean;
		onAlertClick?: (alertId: string) => void;
	}

	let { alerts = [], loading = false, onAlertClick }: Props = $props();

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

<Card>
	<CardHeader>
		<CardTitle class="flex items-center gap-2">
			<AlertTriangle class="h-5 w-5" />
			Alertas Recientes
		</CardTitle>
	</CardHeader>
	<CardContent>
		{#if loading}
			<div class="space-y-3">
				{#each Array(5) as _}
					<div class="flex items-center gap-3">
						<Skeleton class="h-9 w-9 rounded-full" />
						<div class="flex-1 space-y-1.5">
							<Skeleton class="h-4 w-3/4" />
							<Skeleton class="h-3 w-1/2" />
						</div>
					</div>
				{/each}
			</div>
		{:else if alerts.length === 0}
			<div class="flex flex-col items-center justify-center py-6 text-center">
				<CheckCircle class="h-10 w-10 text-green-500 mb-2" />
				<p class="text-sm text-muted-foreground">Sin alertas pendientes</p>
			</div>
		{:else}
			<div class="space-y-3">
				{#each alerts.slice(0, 5) as alert}
					<button
						type="button"
						class="flex w-full items-center gap-3 rounded-lg border p-3 text-left transition-colors hover:bg-accent"
						onclick={() => onAlertClick?.(alert.id)}
					>
						<div
							class={cn(
								'flex h-9 w-9 items-center justify-center rounded-full',
								getSeverityColor(alert.severity)
							)}
						>
							<AlertTriangle class="h-4 w-4" />
						</div>
						<div class="flex-1 space-y-1">
							<p class="text-sm font-medium leading-none">{alert.message}</p>
							<div class="flex items-center gap-2">
								<Clock class="h-3 w-3 text-muted-foreground" />
								<span class="text-xs text-muted-foreground">
									{formatTimeAgo(alert.createdAt)}
								</span>
							</div>
						</div>
						<Badge class={cn('text-xs', getSeverityColor(alert.severity))}>
							{alert.severity}
						</Badge>
					</button>
				{/each}
			</div>
			{#if alerts.length > 5}
				<a
					href="/dashboard/alerts"
					class="mt-4 block text-center text-sm text-primary hover:underline"
				>
					Ver todas las alertas ({alerts.length})
				</a>
			{/if}
		{/if}
	</CardContent>
</Card>
