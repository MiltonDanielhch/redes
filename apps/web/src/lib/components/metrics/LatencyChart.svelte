<!-- Ubicación: `apps/web/src/lib/components/metrics/LatencyChart.svelte` -->
<!-- Descripción: Gráfico de línea para latencia con threshold visual -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';

	interface DataPoint {
		timestamp: string;
		latency_ms: number;
		packet_loss_pct?: number;
	}

	interface Props {
		title?: string;
		data?: DataPoint[];
		threshold?: number;
		loading?: boolean;
	}

	let {
		title = 'Latencia',
		data = [],
		threshold = 100,
		loading = false
	}: Props = $props();

	function formatTime(timestamp: string) {
		const date = new Date(timestamp);
		return date.toLocaleTimeString('es-BO', { hour: '2-digit', minute: '2-digit' });
	}

	let maxLatency = $derived(Math.max(...data.map((d) => d.latency_ms), 1));
	let thresholdY = $derived(100 - (threshold / maxLatency) * 100);

	let points = $derived(
		data.map((d, i) => ({
			...d,
			x: (i / Math.max(data.length - 1, 1)) * 100,
			latencyY: 100 - (d.latency_ms / maxLatency) * 100,
		}))
	);
	let pathD = $derived(
		points.length > 0 ? `M ${points.map((p) => `${p.x},${p.latencyY}`).join(' L ')}` : ''
	);
</script>

<Card>
	<CardHeader>
		<CardTitle class="text-base">{title}</CardTitle>
	</CardHeader>
	<CardContent>
		{#if loading}
			<div class="h-48 animate-pulse rounded bg-muted" />
		{:else if data.length === 0}
			<div class="flex h-48 items-center justify-center text-muted-foreground">
				Sin datos disponibles
			</div>
		{:else}
			<div class="relative h-48">
				<svg class="h-full w-full" viewBox="0 0 100 100" preserveAspectRatio="none">
					{#if thresholdY > 0 && thresholdY < 100}
						<line
							x1="0"
							y1={thresholdY}
							x2="100"
							y2={thresholdY}
							stroke="#ef4444"
							stroke-width="0.5"
							stroke-dasharray="2,2"
							vector-effect="non-scaling-stroke"
						/>
						<text x="98" y={thresholdY - 1} fill="#ef4444" font-size="3" text-anchor="end">
							{threshold}ms
						</text>
					{/if}

					<path
						d={pathD}
						fill="none"
						stroke="#3b82f6"
						stroke-width="0.5"
						vector-effect="non-scaling-stroke"
					/>
				</svg>

				<div class="absolute bottom-0 left-0 right-0 flex justify-between text-xs text-muted-foreground">
					{#if points.length > 0}
						<span>{formatTime(points[0].timestamp)}</span>
						<span>{formatTime(points[points.length - 1].timestamp)}</span>
					{/if}
				</div>
			</div>

			<div class="mt-2 flex justify-between text-sm">
				<span class="text-muted-foreground">
					Min: {Math.min(...data.map((d) => d.latency_ms)).toFixed(0)}ms
				</span>
				<span class="font-medium text-blue-500">
					Max: {Math.max(...data.map((d) => d.latency_ms)).toFixed(0)}ms
				</span>
			</div>
		{/if}
	</CardContent>
</Card>
