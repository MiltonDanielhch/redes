<!-- Ubicación: `apps/web/src/lib/components/dashboard/NetworkHealth.svelte` -->
<!-- Descripción: Muestra estado general de la red con métricas de latencia, packet loss y tendencias -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { Activity, Wifi, WifiOff, TrendingUp } from 'lucide-svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { cn } from '$lib/utils';

	interface Props {
		latencyMs?: number;
		packetLossPct?: number;
		uptimePct?: number;
		trend?: 'improving' | 'degrading' | 'stable';
		loading?: boolean;
	}

	let {
		latencyMs = 0,
		packetLossPct = 0,
		uptimePct = 99.9,
		trend = 'stable',
		loading = false
	}: Props = $props();

	function getHealthStatus() {
		if (latencyMs < 50 && packetLossPct < 1) return { label: 'Excelente', color: 'text-green-500' };
		if (latencyMs < 100 && packetLossPct < 2) return { label: 'Bueno', color: 'text-yellow-500' };
		if (latencyMs < 200 && packetLossPct < 5) return { label: 'Regular', color: 'text-orange-500' };
		return { label: 'Crítico', color: 'text-red-500' };
	}

	function getTrendIcon() {
		switch (trend) {
			case 'improving':
				return TrendingUp;
			case 'degrading':
				return Activity;
			default:
				return Activity;
		}
	}

	const healthStatus = $derived(getHealthStatus());
	const TrendIcon = $derived(getTrendIcon());
</script>

<Card>
	<CardHeader>
		<CardTitle class="flex items-center gap-2">
			<Activity class="h-5 w-5" />
			Estado de la Red
		</CardTitle>
	</CardHeader>
	<CardContent>
		{#if loading}
			<div class="space-y-4">
				<Skeleton class="h-20 w-full" />
				<div class="grid grid-cols-3 gap-4">
					<Skeleton class="h-16 w-full" />
					<Skeleton class="h-16 w-full" />
					<Skeleton class="h-16 w-full" />
				</div>
			</div>
		{:else}
			<div class="space-y-4">
				<div class="flex items-center justify-between rounded-lg border p-4">
					<div class="flex items-center gap-3">
						<div class={cn('flex h-10 w-10 items-center justify-center rounded-full bg-primary/10')}>
							<Wifi class="h-5 w-5" />
						</div>
						<div>
							<p class="text-sm font-medium">Salud General</p>
							<p class={cn('text-2xl font-bold', healthStatus.color)}>
								{healthStatus.label}
							</p>
						</div>
					</div>
					<div class={cn('flex items-center gap-1 text-sm', trend === 'improving' ? 'text-green-500' : trend === 'degrading' ? 'text-red-500' : 'text-muted-foreground')}>
						<TrendIcon class="h-4 w-4" />
						<span>{trend === 'improving' ? 'Mejorando' : trend === 'degrading' ? 'Empeorando' : 'Estable'}</span>
					</div>
				</div>

				<div class="grid grid-cols-3 gap-4">
					<div class="space-y-1 rounded-lg border p-3">
						<p class="text-xs text-muted-foreground">Latencia</p>
						<p class={cn('text-xl font-bold', latencyMs < 100 ? 'text-green-500' : latencyMs < 200 ? 'text-yellow-500' : 'text-red-500')}>
							{latencyMs} ms
						</p>
					</div>
					<div class="space-y-1 rounded-lg border p-3">
						<p class="text-xs text-muted-foreground">Packet Loss</p>
						<p class={cn('text-xl font-bold', packetLossPct < 1 ? 'text-green-500' : packetLossPct < 5 ? 'text-yellow-500' : 'text-red-500')}>
							{packetLossPct}%
						</p>
					</div>
					<div class="space-y-1 rounded-lg border p-3">
						<p class="text-xs text-muted-foreground">Uptime</p>
						<p class="text-xl font-bold text-green-500">
							{uptimePct}%
						</p>
					</div>
				</div>
			</div>
		{/if}
	</CardContent>
</Card>
