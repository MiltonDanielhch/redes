<!-- Ubicación: `apps/web/src/lib/components/metrics/MetricsSummary.svelte` -->
<!-- Descripción: Panel de resumen con stats de métricas y comparación -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { TrendingUp, TrendingDown, Minus, AlertTriangle } from 'lucide-svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';

	interface Props {
		avg?: number;
		min?: number;
		max?: number;
		p95?: number;
		previousAvg?: number;
		anomalyDetected?: boolean;
		loading?: boolean;
	}

	let {
		avg = 0,
		min = 0,
		max = 0,
		p95 = 0,
		previousAvg = 0,
		anomalyDetected = false,
		loading = false
	}: Props = $props();

	let delta = $derived(
		previousAvg > 0 ? ((avg - previousAvg) / previousAvg) * 100 : 0
	);
	let trendIcon = $derived(
		delta > 5 ? TrendingUp : delta < -5 ? TrendingDown : Minus
	);
	let trendColor = $derived(
		delta > 5 ? 'text-green-500' : delta < -5 ? 'text-red-500' : 'text-muted-foreground'
	);
</script>

<Card>
	<CardHeader class="pb-2">
		<CardTitle class="flex items-center justify-between text-base">
			<span>Resumen de Métricas</span>
			{#if anomalyDetected}
				<Badge variant="destructive" class="text-xs">
					<AlertTriangle class="mr-1 h-3 w-3" />
					Anomalía
				</Badge>
			{/if}
		</CardTitle>
	</CardHeader>
	<CardContent>
		{#if loading}
			<div class="grid grid-cols-2 gap-4">
				{#each Array(4) as _}
					<div class="h-16 animate-pulse rounded bg-muted" />
				{/each}
			</div>
		{:else}
			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-1">
					<p class="text-xs text-muted-foreground">Promedio</p>
					<p class="text-xl font-bold">{avg.toFixed(2)}</p>
					<div class="flex items-center gap-1 text-xs {trendColor}">
						<svelte:component this={trendIcon} class="h-3 w-3" />
						<span>{delta > 0 ? '+' : ''}{delta.toFixed(1)}% vs anterior</span>
					</div>
				</div>

				<div class="space-y-1">
					<p class="text-xs text-muted-foreground">Pico (P95)</p>
					<p class="text-xl font-bold">{p95.toFixed(2)}</p>
					<p class="text-xs text-muted-foreground">percentil 95</p>
				</div>

				<div class="space-y-1">
					<p class="text-xs text-muted-foreground">Mínimo</p>
					<p class="text-xl font-bold">{min.toFixed(2)}</p>
					<p class="text-xs text-muted-foreground">valor más bajo</p>
				</div>

				<div class="space-y-1">
					<p class="text-xs text-muted-foreground">Máximo</p>
					<p class="text-xl font-bold">{max.toFixed(2)}</p>
					<p class="text-xs text-muted-foreground">valor más alto</p>
				</div>
			</div>
		{/if}
	</CardContent>
</Card>
