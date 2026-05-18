<!-- Ubicación: `apps/web/src/lib/components/agents/AgentCard.svelte` -->
<!-- Descripción: Card de агент con estado, métricas y acciones rápidas -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0022 (Agentes) -->
<script lang="ts">
	import { Server, CheckCircle, XCircle, RefreshCw, Play, Pause, RotateCw } from 'lucide-svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { cn } from '$lib/utils';
	import type { AgentResponse } from '$lib/generated/api-types';

	interface Props {
		agent?: AgentResponse;
		metricsCollected?: number;
		errorCount?: number;
		onRestart?: (agentId: string) => void;
		onPause?: (agentId: string) => void;
		onResume?: (agentId: string) => void;
	}

	let {
		agent,
		metricsCollected = 0,
		errorCount = 0,
		onRestart,
		onPause,
		onResume
	}: Props = $props();

	function getStatusColor(status: string) {
		switch (status.toLowerCase()) {
			case 'online':
			case 'active':
				return 'bg-green-500/10 text-green-500 border-green-500/20';
			case 'offline':
				return 'bg-red-500/10 text-red-500 border-red-500/20';
			case 'paused':
			case 'syncing':
				return 'bg-yellow-500/10 text-yellow-500 border-yellow-500/20';
			default:
				return 'bg-gray-500/10 text-gray-500 border-gray-500/20';
		}
	}

	function getStatusIcon(status: string) {
		switch (status.toLowerCase()) {
			case 'online':
			case 'active':
				return CheckCircle;
			case 'offline':
				return XCircle;
			default:
				return RefreshCw;
		}
	}

	function formatLastSeen(dateStr: string) {
		const date = new Date(dateStr);
		const now = new Date();
		const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

		if (seconds < 60) return 'Hace un momento';
		if (seconds < 3600) return `Hace ${Math.floor(seconds / 60)} min`;
		if (seconds < 86400) return `Hace ${Math.floor(seconds / 3600)}h`;
		return `Hace ${Math.floor(seconds / 86400)}d`;
	}

	const StatusIcon = $derived(agent ? getStatusIcon(agent.status) : RefreshCw);
</script>

<Card class="relative overflow-hidden">
	<CardHeader class="pb-2">
		<div class="flex items-start justify-between">
			<div class="flex items-center gap-3">
				<div class={cn('flex h-10 w-10 items-center justify-center rounded-lg', getStatusColor(agent?.status ?? 'offline'))}>
					<Server class="h-5 w-5" />
				</div>
				<div>
					<CardTitle class="text-base">{agent?.name ?? 'Sin nombre'}</CardTitle>
					<p class="text-xs text-muted-foreground">{agent?.hostname ?? 'N/A'}</p>
				</div>
			</div>
			<Badge class={cn('text-xs', getStatusColor(agent?.status ?? 'offline'))}>
				<svelte:component this={StatusIcon} class="mr-1 h-3 w-3" />
				{agent?.status ?? 'offline'}
			</Badge>
		</div>
	</CardHeader>

	<CardContent class="space-y-4">
		<div class="grid grid-cols-2 gap-4 text-sm">
			<div>
				<p class="text-muted-foreground">IP:Puerto</p>
				<p class="font-medium">{agent?.ip_address ?? 'N/A'}:{agent?.port ?? 'N/A'}</p>
			</div>
			<div>
				<p class="text-muted-foreground">Tipo</p>
				<p class="font-medium">{agent?.agent_type ?? 'N/A'}</p>
			</div>
			<div>
				<p class="text-muted-foreground">Última conexión</p>
				<p class="font-medium">{agent ? formatLastSeen(agent.last_seen_at) : 'Nunca'}</p>
			</div>
			<div>
				<p class="text-muted-foreground">Versión</p>
				<p class="font-medium">{agent?.version ?? 'N/A'}</p>
			</div>
		</div>

		<div class="flex gap-2 border-t pt-4">
			{#if agent?.status === 'active'}
				<Button variant="outline" size="sm" class="flex-1" onclick={() => onPause?.(agent!.id)}>
					<Pause class="mr-1 h-4 w-4" />
					Pausar
				</Button>
			{:else if agent?.status === 'paused'}
				<Button variant="outline" size="sm" class="flex-1" onclick={() => onResume?.(agent!.id)}>
					<Play class="mr-1 h-4 w-4" />
					Reanudar
				</Button>
			{/if}
			<Button variant="outline" size="sm" onclick={() => onRestart?.(agent!.id)}>
				<RotateCw class="h-4 w-4" />
			</Button>
		</div>
	</CardContent>
</Card>
