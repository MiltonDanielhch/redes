<!-- Ubicación: `apps/web/src/lib/components/metrics/MetricsChart.svelte` -->
<!-- Descripción: Gráfico de área/línea para métricas de ancho de banda -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';

	interface DataPoint {
		timestamp: string;
		value: number;
	}

	interface Props {
		title?: string;
		data?: DataPoint[];
		color?: string;
		unit?: string;
		loading?: boolean;
	}

	let {
		title = 'Métricas',
		data = [],
		color = '#3b82f6',
		unit = '',
		loading = false
	}: Props = $props();

	function formatTime(timestamp: string) {
		const date = new Date(timestamp);
		return date.toLocaleTimeString('es-BO', { hour: '2-digit', minute: '2-digit' });
	}

	function formatValue(value: number) {
		if (value >= 1000) {
			return `${(value / 1000).toFixed(1)} Gbps`;
		}
		return `${value.toFixed(1)} ${unit}`;
	}

	let maxValue = $derived(Math.max(...data.map((d) => d.value), 1));
	let points = $derived(
		data.map((d, i) => ({
			...d,
			x: (i / Math.max(data.length - 1, 1)) * 100,
			y: 100 - (d.value / maxValue) * 100,
		}))
	);
	let pathD = $derived(
		points.length > 0
			? `M ${points.map((p) => `${p.x},${p.y}`).join(' L ')}`
			: ''
	);
	let areaD = $derived(
		points.length > 0 ? `${pathD} L 100,100 L 0,100 Z` : ''
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
					<defs>
						<linearGradient id="gradient-{title}" x1="0" y1="0" x2="0" y2="1">
							<stop offset="0%" stop-color={color} stop-opacity="0.3" />
							<stop offset="100%" stop-color={color} stop-opacity="0" />
						</linearGradient>
					</defs>
					<path d={areaD} fill="url(#gradient-{title})" />
					<path d={pathD} fill="none" stroke={color} stroke-width="0.5" vector-effect="non-scaling-stroke" />
				</svg>

				<div class="absolute bottom-0 left-0 right-0 flex justify-between text-xs text-muted-foreground">
					{#if points.length > 0}
						<span>{formatTime(points[0].timestamp)}</span>
						<span>{formatTime(points[points.length - 1].timestamp)}</span>
					{/if}
				</div>
			</div>

			<div class="mt-2 flex justify-between text-sm">
				<span class="text-muted-foreground">Min: {formatValue(Math.min(...data.map((d) => d.value)))}</span>
				<span class="font-medium" style="color: {color}">Max: {formatValue(Math.max(...data.map((d) => d.value)))}</span>
			</div>
		{/if}
	</CardContent>
</Card>
