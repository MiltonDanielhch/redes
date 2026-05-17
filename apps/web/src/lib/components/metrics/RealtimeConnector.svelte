<!-- Ubicación: `apps/web/src/lib/components/metrics/RealtimeConnector.svelte` -->
<!-- Descripción: Connector SSE para métricas en tiempo real con reconexión automática -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0021 (Offline), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { toast } from 'svelte-sonner';

	interface MetricUpdate {
		type: 'metric_update';
		device_id: string;
		metric_type: string;
		value: number;
		timestamp: string;
	}

	interface AlertUpdate {
		type: 'alert_new';
		alert: {
			id: string;
			severity: string;
			message: string;
			created_at: string;
		};
	}

	interface DeviceStatusChange {
		type: 'device_status_change';
		device_id: string;
		old_status: string;
		new_status: string;
	}

	interface Props {
		deviceId?: string;
		sedeId?: string;
		onMetric?: (metric: MetricUpdate) => void;
		onAlert?: (alert: AlertUpdate) => void;
		onDeviceStatus?: (status: DeviceStatusChange) => void;
	}

	let {
		deviceId,
		sedeId,
		onMetric,
		onAlert,
		onDeviceStatus
	}: Props = $props();

	let eventSource: EventSource | null = null;
	let reconnectAttempts = $state(0);
	let isConnected = $state(false);
	let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;

	const MAX_RECONNECT_ATTEMPTS = 10;
	const RECONNECT_BASE_DELAY = 1000;

	function getReconnectDelay(): number {
		return Math.min(RECONNECT_BASE_DELAY * Math.pow(2, reconnectAttempts), 30000);
	}

	function connect() {
		if (typeof window === 'undefined') return;

		const params = new URLSearchParams();
		if (deviceId) params.set('device_id', deviceId);
		if (sedeId) params.set('sedes', sedeId);

		const url = `${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/stream/metrics?${params}`;

		try {
			eventSource = new EventSource(url);

			eventSource.onopen = () => {
				isConnected = true;
				reconnectAttempts = 0;
			};

			eventSource.onerror = () => {
				isConnected = false;
				eventSource?.close();

				if (reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
					const delay = getReconnectDelay();
					reconnectTimeout = setTimeout(() => {
						reconnectAttempts++;
						connect();
					}, delay);
				} else {
					toast.error('Conexión en tiempo real perdida. Usando modo polling.');
				}
			};

			eventSource.addEventListener('metric_update', (e) => {
				try {
					const data: MetricUpdate = JSON.parse(e.data);
					onMetric?.(data);
				} catch {
					console.error('Failed to parse metric update');
				}
			});

			eventSource.addEventListener('alert_new', (e) => {
				try {
					const data: AlertUpdate = JSON.parse(e.data);
					onAlert?.(data);
					toast.warning(`Nueva alerta: ${data.alert.message}`);
				} catch {
					console.error('Failed to parse alert');
				}
			});

			eventSource.addEventListener('device_status_change', (e) => {
				try {
					const data: DeviceStatusChange = JSON.parse(e.data);
					onDeviceStatus?.(data);
				} catch {
					console.error('Failed to parse device status');
				}
			});
		} catch (error) {
			console.error('Failed to create EventSource:', error);
		}
	}

	onMount(() => {
		connect();
	});

	onDestroy(() => {
		if (eventSource) {
			eventSource.close();
		}
		if (reconnectTimeout) {
			clearTimeout(reconnectTimeout);
		}
	});
</script>

<div class="flex items-center gap-2 text-sm text-muted-foreground">
	{#if isConnected}
		<div class="h-2 w-2 rounded-full bg-green-500 animate-pulse" />
		<span>Conectado</span>
	{:else}
		<div class="h-2 w-2 rounded-full bg-yellow-500 animate-pulse" />
		<span>Reconectando... ({reconnectAttempts}/{MAX_RECONNECT_ATTEMPTS})</span>
	{/if}
</div>
