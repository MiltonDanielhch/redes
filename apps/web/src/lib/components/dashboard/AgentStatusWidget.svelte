<!-- Ubicación: `apps/web/src/lib/components/dashboard/AgentStatusWidget.svelte` -->
<!-- Descripción: Widget compacto que muestra estado de agentes de monitoreo (online, offline, syncing) -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0022 (Agentes) -->
<script lang="ts">
	import { Server, CheckCircle, XCircle, RefreshCw } from 'lucide-svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { cn } from '$lib/utils';

	interface AgentInfo {
		id: string;
		name: string;
		hostname: string;
		status: 'online' | 'offline' | 'syncing';
		lastSeenAt: string;
	}

	interface Props {
		agents?: AgentInfo[];
		loading?: boolean;
		onAgentClick?: (agentId: string) => void;
	}

	let { agents = [], loading = false, onAgentClick }: Props = $props();

	function getStatusIcon(status: string) {
		switch (status) {
			case 'online':
				return CheckCircle;
			case 'offline':
				return XCircle;
			case 'syncing':
				return RefreshCw;
			default:
				return Server;
		}
	}

	function getStatusColor(status: string) {
		switch (status) {
			case 'online':
				return 'bg-green-500/10 text-green-500 border-green-500/20';
			case 'offline':
				return 'bg-red-500/10 text-red-500 border-red-500/20';
			case 'syncing':
				return 'bg-yellow-500/10 text-yellow-500 border-yellow-500/20';
			default:
				return 'bg-gray-500/10 text-gray-500 border-gray-500/20';
		}
	}

	function getStatusLabel(status: string) {
		switch (status) {
			case 'online':
				return 'Online';
			case 'offline':
				return 'Offline';
			case 'syncing':
				return 'Sincronizando';
			default:
				return status;
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

	let onlineCount = $derived(agents.filter((a) => a.status === 'online').length);
	let offlineCount = $derived(agents.filter((a) => a.status === 'offline').length);
</script>

<Card>
	<CardHeader>
		<CardTitle class="flex items-center gap-2">
			<Server class="h-5 w-5" />
			Agentes de Monitoreo
		</CardTitle>
	</CardHeader>
	<CardContent>
		{#if loading}
			<div class="space-y-3">
				{#each Array(4) as _}
					<div class="flex items-center gap-3">
						<Skeleton class="h-9 w-9 rounded-full" />
						<div class="flex-1 space-y-1">
							<Skeleton class="h-4 w-1/2" />
							<Skeleton class="h-3 w-1/3" />
						</div>
					</div>
				{/each}
			</div>
		{:else if agents.length === 0}
			<p class="py-6 text-center text-sm text-muted-foreground">
				No hay agentes registrados
			</p>
		{:else}
			<div class="space-y-3">
				{#each agents.slice(0, 5) as agent}
					{@const StatusIcon = getStatusIcon(agent.status)}
					<button
						type="button"
						class="flex w-full items-center gap-3 rounded-lg border p-3 text-left transition-colors hover:bg-accent"
						onclick={() => onAgentClick?.(agent.id)}
					>
						<div class={cn('flex h-9 w-9 items-center justify-center rounded-full', getStatusColor(agent.status))}>
							<StatusIcon class="h-4 w-4" />
						</div>
						<div class="flex-1 space-y-1">
							<p class="text-sm font-medium">{agent.name}</p>
							<p class="text-xs text-muted-foreground">
								{agent.hostname} • {formatLastSeen(agent.lastSeenAt)}
							</p>
						</div>
						<Badge class={cn('text-xs', getStatusColor(agent.status))}>
							{getStatusLabel(agent.status)}
						</Badge>
					</button>
				{/each}
			</div>

			<div class="mt-4 flex justify-between border-t pt-4 text-sm">
				<div class="flex items-center gap-1">
					<div class="h-2 w-2 rounded-full bg-green-500" />
					<span class="text-muted-foreground">{onlineCount} online</span>
				</div>
				<div class="flex items-center gap-1">
					<div class="h-2 w-2 rounded-full bg-red-500" />
					<span class="text-muted-foreground">{offlineCount} offline</span>
				</div>
			</div>

			{#if agents.length > 5}
				<a
					href="/dashboard/agents"
					class="mt-4 block text-center text-sm text-primary hover:underline"
				>
					Ver todos los agentes ({agents.length})
				</a>
			{/if}
		{/if}
	</CardContent>
</Card>
