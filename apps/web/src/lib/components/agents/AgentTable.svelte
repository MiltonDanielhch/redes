<!-- Ubicación: `apps/web/src/lib/components/agents/AgentTable.svelte` -->
<!-- Descripción: Tabla de агентs с состоянием и действиями -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0022 (Agentes) -->
<script lang="ts">
	import { Server, CheckCircle, XCircle, RefreshCw } from 'lucide-svelte';
	import { Card, CardContent } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { cn } from '$lib/utils';
	import type { AgentResponse } from '$lib/generated/api-types';

	interface Props {
		agents?: AgentResponse[];
		loading?: boolean;
		onRestart?: (agentId: string) => void;
		onPause?: (agentId: string) => void;
		onResume?: (agentId: string) => void;
	}

	let { agents = [], loading = false, onRestart, onPause, onResume }: Props = $props();

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
</script>

<div class="space-y-4">
	{#if loading}
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each Array(6) as _}
				<Card>
					<CardContent class="p-6">
						<div class="h-32 animate-pulse rounded bg-muted" />
					</CardContent>
				</Card>
			{/each}
		</div>
	{:else if agents.length === 0}
		<Card>
			<CardContent class="flex flex-col items-center justify-center py-12">
				<Server class="h-12 w-12 text-muted-foreground mb-4" />
				<p class="text-lg font-medium">No hay agentes registrados</p>
				<p class="text-muted-foreground">Los agentes se registrarán automáticamente al conectarse</p>
			</CardContent>
		</Card>
	{:else}
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each agents as agent (agent.id)}
				{@const StatusIcon = getStatusIcon(agent.status)}
				<Card class="relative overflow-hidden">
					<CardContent class="p-4">
						<div class="flex items-start justify-between mb-3">
							<div class="flex items-center gap-3">
								<div class={cn('flex h-10 w-10 items-center justify-center rounded-lg', getStatusColor(agent.status))}>
									<Server class="h-5 w-5" />
								</div>
								<div>
									<p class="font-medium">{agent.name}</p>
									<p class="text-xs text-muted-foreground">{agent.hostname}</p>
								</div>
							</div>
							<Badge class={cn('text-xs', getStatusColor(agent.status))}>
								{agent.status}
							</Badge>
						</div>

						<div class="grid grid-cols-2 gap-2 text-xs">
							<div>
								<span class="text-muted-foreground">IP:Puerto</span>
								<p class="font-medium">{agent.ip_address}:{agent.port}</p>
							</div>
							<div>
								<span class="text-muted-foreground">Tipo</span>
								<p class="font-medium">{agent.agent_type}</p>
							</div>
							<div>
								<span class="text-muted-foreground">Última conexión</span>
								<p class="font-medium">{formatLastSeen(agent.last_seen_at)}</p>
							</div>
							<div>
								<span class="text-muted-foreground">Versión</span>
								<p class="font-medium">{agent.version}</p>
							</div>
						</div>

						<div class="mt-3 flex gap-2 border-t pt-3">
							{#if agent.status === 'active'}
								<Button variant="outline" size="sm" class="flex-1" onclick={() => onPause?.(agent.id)}>
									Pausar
								</Button>
							{:else if agent.status === 'paused'}
								<Button variant="outline" size="sm" class="flex-1" onclick={() => onResume?.(agent.id)}>
									Reanudar
								</Button>
							{/if}
							<Button variant="outline" size="sm" onclick={() => onRestart?.(agent.id)}>
								<RefreshCw class="h-4 w-4" />
							</Button>
						</div>
					</CardContent>
				</Card>
			{/each}
		</div>
	{/if}
</div>
