<!-- Ubicación: `apps/web/src/routes/(dashboard)/agents/+page.svelte` -->
<!-- Descripción: Página de gestión de agentes distribuidos con tabla y acciones -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0022 (Agentes) -->
<script lang="ts">
	import AgentTable from '$lib/components/agents/AgentTable.svelte';
	import { createQuery, useQueryClient } from '@tanstack/svelte-query';
	import { toast } from 'svelte-sonner';
	import { Server, RefreshCw } from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button';

	const queryClient = useQueryClient();

	let agentsQuery = createQuery({
		queryKey: ['agents'],
		queryFn: async () => {
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/agents`);
			if (!response.ok) throw new Error('Failed to fetch agents');
			return response.json();
		},
		staleTime: 30 * 1000,
		refetchInterval: 60 * 1000,
	});

	async function handleRestart(agentId: string) {
		try {
			const response = await fetch(
				`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/agents/${agentId}/restart`,
				{ method: 'POST' }
			);
			if (!response.ok) throw new Error('Failed to restart agent');
			toast.success('Agente reiniciado');
			queryClient.invalidateQueries({ queryKey: ['agents'] });
		} catch {
			toast.error('Error al reiniciar agente');
		}
	}

	async function handlePause(agentId: string) {
		try {
			const response = await fetch(
				`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/agents/${agentId}/pause`,
				{ method: 'POST' }
			);
			if (!response.ok) throw new Error('Failed to pause agent');
			toast.success('Agente pausado');
			queryClient.invalidateQueries({ queryKey: ['agents'] });
		} catch {
			toast.error('Error al pausar agente');
		}
	}

	async function handleResume(agentId: string) {
		try {
			const response = await fetch(
				`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/agents/${agentId}/resume`,
				{ method: 'POST' }
			);
			if (!response.ok) throw new Error('Failed to resume agent');
			toast.success('Agente reanudado');
			queryClient.invalidateQueries({ queryKey: ['agents'] });
		} catch {
			toast.error('Error al reanudar agente');
		}
	}

	let onlineCount = $derived(($agentsQuery.data ?? []).filter((a: { status: string }) => a.status === 'online').length);
	let offlineCount = $derived(($agentsQuery.data ?? []).filter((a: { status: string }) => a.status === 'offline').length);
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold tracking-tight">Agentes de Monitoreo</h1>
			<p class="text-muted-foreground">Gestión de agentes SNMP y Netflow distribuidos</p>
		</div>
		<Button variant="outline" onclick={() => queryClient.invalidateQueries({ queryKey: ['agents'] })}>
			<RefreshCw class="mr-2 h-4 w-4" />
			Actualizar
		</Button>
	</div>

	<div class="grid gap-4 md:grid-cols-3">
		<div class="rounded-lg border bg-card p-4">
			<div class="flex items-center gap-4">
				<div class="flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
					<Server class="h-6 w-6 text-primary" />
				</div>
				<div>
					<p class="text-sm text-muted-foreground">Total Agentes</p>
					<p class="text-2xl font-bold">{$agentsQuery.data?.length ?? 0}</p>
				</div>
			</div>
		</div>

		<div class="rounded-lg border bg-card p-4">
			<div class="flex items-center gap-4">
				<div class="flex h-12 w-12 items-center justify-center rounded-full bg-green-500/10">
					<Server class="h-6 w-6 text-green-500" />
				</div>
				<div>
					<p class="text-sm text-muted-foreground">Online</p>
					<p class="text-2xl font-bold text-green-500">{onlineCount}</p>
				</div>
			</div>
		</div>

		<div class="rounded-lg border bg-card p-4">
			<div class="flex items-center gap-4">
				<div class="flex h-12 w-12 items-center justify-center rounded-full bg-red-500/10">
					<Server class="h-6 w-6 text-red-500" />
				</div>
				<div>
					<p class="text-sm text-muted-foreground">Offline</p>
					<p class="text-2xl font-bold text-red-500">{offlineCount}</p>
				</div>
			</div>
		</div>
	</div>

	<AgentTable
		agents={$agentsQuery.data}
		loading={$agentsQuery.isLoading}
		onRestart={handleRestart}
		onPause={handlePause}
		onResume={handleResume}
	/>
</div>
