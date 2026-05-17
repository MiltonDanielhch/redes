<!-- Ubicación: `apps/web/src/routes/(dashboard)/metrics/+page.svelte` -->
<!-- Descripción: Página de métricas con gráficos, selector de rango y modo realtime -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { RefreshCw, Wifi, WifiOff } from 'lucide-svelte';
	import MetricsChart from '$lib/components/metrics/MetricsChart.svelte';
	import LatencyChart from '$lib/components/metrics/LatencyChart.svelte';
	import MetricsSummary from '$lib/components/metrics/MetricsSummary.svelte';
	import RealtimeConnector from '$lib/components/metrics/RealtimeConnector.svelte';
	import { Button } from '$lib/components/ui/button';
	import { createQuery } from '@tanstack/svelte-query';

	interface DataPoint {
		timestamp: string;
		value: number;
	}

	let selectedDeviceId = $state<string | null>(null);
	let selectedRange = $state<'1h' | '6h' | '24h' | '7d' | '30d'>('24h');
	let isRealtime = $state(true);

	let devicesQuery = createQuery({
		queryKey: ['devices'],
		queryFn: async () => {
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/devices`);
			if (!response.ok) throw new Error('Failed to fetch devices');
			return response.json();
		},
	});

	let metricsQuery = createQuery({
		queryKey: ['metrics', selectedDeviceId, selectedRange],
		queryFn: async () => {
			const params = new URLSearchParams();
			if (selectedDeviceId) params.set('device_id', selectedDeviceId);
			params.set('range', selectedRange);
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/metrics?${params}`);
			if (!response.ok) throw new Error('Failed to fetch metrics');
			return response.json();
		},
		enabled: !!selectedDeviceId,
		refetchInterval: isRealtime ? 30000 : 0,
	});

	let realtimeMetrics = $state<{
		bandwidth_rx: DataPoint[];
		bandwidth_tx: DataPoint[];
		latency_ms: DataPoint[];
	}>({
		bandwidth_rx: [],
		bandwidth_tx: [],
		latency_ms: [],
	});

	function handleMetricUpdate(metric: { device_id: string; metric_type: string; value: number; timestamp: string }) {
		if (metric.device_id !== selectedDeviceId) return;

		switch (metric.metric_type) {
			case 'bandwidth_rx':
				realtimeMetrics.bandwidth_rx = [
					...realtimeMetrics.bandwidth_rx.slice(-50),
					{ timestamp: metric.timestamp, value: metric.value }
				];
				break;
			case 'bandwidth_tx':
				realtimeMetrics.bandwidth_tx = [
					...realtimeMetrics.bandwidth_tx.slice(-50),
					{ timestamp: metric.timestamp, value: metric.value }
				];
				break;
			case 'latency_ms':
				realtimeMetrics.latency_ms = [
					...realtimeMetrics.latency_ms.slice(-50),
					{ timestamp: metric.timestamp, value: metric.value }
				];
				break;
		}
	}

	const ranges = [
		{ value: '1h', label: '1 hora' },
		{ value: '6h', label: '6 horas' },
		{ value: '24h', label: '24 horas' },
		{ value: '7d', label: '7 días' },
		{ value: '30d', label: '30 días' },
	] as const;
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold tracking-tight">Métricas</h1>
			<p class="text-muted-foreground">Monitoreo de rendimiento de red</p>
		</div>

		<div class="flex items-center gap-2">
			<Button
				variant={isRealtime ? 'default' : 'outline'}
				size="sm"
				onclick={() => (isRealtime = !isRealtime)}
			>
				{#if isRealtime}
					<Wifi class="mr-2 h-4 w-4" />
					Realtime
				{:else}
					<WifiOff class="mr-2 h-4 w-4" />
					Polling
				{/if}
			</Button>
		</div>
	</div>

	<div class="flex flex-wrap items-center gap-4">
		<div class="flex items-center gap-2">
			<label class="text-sm font-medium" for="device-select">Dispositivo:</label>
			<select
				id="device-select"
				bind:value={selectedDeviceId}
				class="flex h-10 rounded-md border border-input bg-background px-3 py-2 text-sm"
			>
				<option value={null}>Seleccionar dispositivo</option>
				{#each $devicesQuery.data ?? [] as device}
					<option value={device.id}>{device.hostname}</option>
				{/each}
			</select>
		</div>

		<div class="flex items-center gap-2">
			<span class="text-sm font-medium">Rango:</span>
			<div class="flex rounded-md border">
				{#each ranges as range}
					<button
						type="button"
						class="px-3 py-2 text-sm {selectedRange === range.value
							? 'bg-primary text-primary-foreground'
							: 'hover:bg-accent'}"
						onclick={() => (selectedRange = range.value)}
					>
						{range.label}
					</button>
				{/each}
			</div>
		</div>

		{#if $metricsQuery.isFetching}
			<div class="flex items-center gap-2 text-sm text-muted-foreground">
				<RefreshCw class="h-4 w-4 animate-spin" />
				<span>Actualizando...</span>
			</div>
		{/if}
	</div>

	{#if selectedDeviceId && isRealtime}
		<RealtimeConnector
			deviceId={selectedDeviceId}
			onMetric={handleMetricUpdate}
		/>
	{/if}

	{#if $metricsQuery.data}
		<div class="grid gap-4 md:grid-cols-2">
			<MetricsSummary
				avg={$metricsQuery.data.avg_bandwidth}
				min={$metricsQuery.data.min_bandwidth}
				max={$metricsQuery.data.max_bandwidth}
				p95={$metricsQuery.data.p95_bandwidth}
				previousAvg={$metricsQuery.data.previous_avg_bandwidth}
				anomalyDetected={$metricsQuery.data.anomaly_detected}
				loading={$metricsQuery.isLoading}
			/>

			<LatencyChart
				title="Latencia"
				data={isRealtime ? realtimeMetrics.latency_ms : $metricsQuery.data.latency ?? []}
				threshold={100}
				loading={$metricsQuery.isLoading}
			/>
		</div>

		<div class="grid gap-4 md:grid-cols-2">
			<MetricsChart
				title="Ancho de Banda RX"
				data={isRealtime ? realtimeMetrics.bandwidth_rx : $metricsQuery.data.bandwidth_rx ?? []}
				color="#22c55e"
				unit="Mbps"
				loading={$metricsQuery.isLoading}
			/>

			<MetricsChart
				title="Ancho de Banda TX"
				data={isRealtime ? realtimeMetrics.bandwidth_tx : $metricsQuery.data.bandwidth_tx ?? []}
				color="#3b82f6"
				unit="Mbps"
				loading={$metricsQuery.isLoading}
			/>
		</div>
	{:else if !selectedDeviceId}
		<div class="flex flex-col items-center justify-center py-12 text-center">
			<Wifi class="h-12 w-12 text-muted-foreground mb-4" />
			<p class="text-lg font-medium">Selecciona un dispositivo</p>
			<p class="text-muted-foreground">Elige un dispositivo para ver sus métricas</p>
		</div>
	{:else}
		<div class="flex items-center justify-center py-12">
			<RefreshCw class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{/if}
</div>
