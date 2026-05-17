<!-- Ubicación: `apps/web/src/routes/(dashboard)/topology/+page.svelte` -->
<!-- Descripción: Página de topología de red con mapa interactivo y sidebar -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import TopologyMap from '$lib/components/topology/TopologyMap.svelte';
	import TopologySidebar from '$lib/components/topology/TopologySidebar.svelte';
	import { createQuery } from '@tanstack/svelte-query';
	import { Building2, MapPin } from 'lucide-svelte';

	let selectedNodeId = $state<string | null>(null);
	let selectedSedeId = $state<string | null>(null);

	let sedesQuery = createQuery({
		queryKey: ['sedes', 'with-devices'],
		queryFn: async () => {
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/sedes?include_devices=true`);
			if (!response.ok) throw new Error('Failed to fetch sedes');
			return response.json();
		},
		staleTime: 60 * 1000,
	});

	let topologyQuery = createQuery({
		queryKey: ['topology', selectedSedeId],
		queryFn: async () => {
			const params = new URLSearchParams();
			if (selectedSedeId) params.set('sede_id', selectedSedeId);
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/topology?${params}`);
			if (!response.ok) throw new Error('Failed to fetch topology');
			return response.json();
		},
		enabled: !!selectedSedeId,
		staleTime: 30 * 1000,
	});

	function handleNodeClick(nodeId: string) {
		selectedNodeId = nodeId;
	}

	function handleSelectFromSidebar(nodeId: string, type: 'sede' | 'device') {
		selectedNodeId = nodeId;
		if (type === 'sede') {
			selectedSedeId = nodeId;
		}
	}

	function handleSedeSelect(sedeId: string) {
		selectedSedeId = sedeId;
		selectedNodeId = null;
	}
</script>

<div class="space-y-6">
	<div>
		<h1 class="text-3xl font-bold tracking-tight">Topología de Red</h1>
		<p class="text-muted-foreground">Visualización de la infraestructura de red regional</p>
	</div>

	<div class="grid gap-6 lg:grid-cols-4">
		<div class="lg:col-span-3">
			<TopologyMap
				nodes={$topologyQuery.data?.nodes}
				edges={$topologyQuery.data?.edges}
				selectedNodeId={selectedNodeId}
				onNodeClick={handleNodeClick}
				loading={$topologyQuery.isLoading}
			/>
		</div>

		<div class="lg:col-span-1">
			<div class="space-y-4">
				<div>
					<label for="sede-select" class="mb-2 block text-sm font-medium">
						Seleccionar Sede
					</label>
					<select
						id="sede-select"
						value={selectedSedeId}
						onchange={(e) => handleSedeSelect(e.currentTarget.value)}
						class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
					>
						<option value={null}>Todas las sedes</option>
						{#each $sedesQuery.data ?? [] as sede}
							<option value={sede.id}>{sede.nombre}</option>
						{/each}
					</select>
				</div>

				<TopologySidebar
					sedes={$sedesQuery.data ?? []}
					selectedNodeId={selectedNodeId}
					onSelectNode={handleSelectFromSidebar}
					loading={$sedesQuery.isLoading}
				/>
			</div>
		</div>
	</div>

	{#if selectedNodeId && $topologyQuery.data?.nodes}
		{@const selectedNode = $topologyQuery.data.nodes.find((n: { id: string }) => n.id === selectedNodeId)}
		{#if selectedNode}
			<div class="rounded-lg border bg-card p-4">
				<div class="flex items-center gap-4">
					<div class="flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
						{#if selectedNode.type === 'sede'}
							<Building2 class="h-6 w-6 text-primary" />
						{:else}
							<MapPin class="h-6 w-6 text-primary" />
						{/if}
					</div>
					<div>
						<h3 class="text-lg font-semibold">{selectedNode.label}</h3>
						<p class="text-sm text-muted-foreground">
							Tipo: {selectedNode.type} | Estado: {selectedNode.status}
						</p>
					</div>
				</div>
			</div>
		{/if}
	{/if}
</div>
