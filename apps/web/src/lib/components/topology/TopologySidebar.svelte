<!-- Ubicación: `apps/web/src/lib/components/topology/TopologySidebar.svelte` -->
<!-- Descripción: Sidebar que muestra lista jerárquica de sedes > dispositivos con estado -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { ChevronDown, ChevronRight, Building2, Monitor } from 'lucide-svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { cn } from '$lib/utils';

	interface Device {
		id: string;
		hostname: string;
		status: string;
	}

	interface Sede {
		id: string;
		nombre: string;
		ubicacion: string;
		device_count?: number;
		devices?: Device[];
	}

	interface Props {
		sedes?: Sede[];
		selectedNodeId?: string | null;
		onSelectNode?: (nodeId: string, type: 'sede' | 'device') => void;
		loading?: boolean;
	}

	let { sedes = [], selectedNodeId = null, onSelectNode, loading = false }: Props = $props();

	let expandedSedes = $state<Set<string>>(new Set());

	function toggleSede(sedeId: string) {
		const newExpanded = new Set(expandedSedes);
		if (newExpanded.has(sedeId)) {
			newExpanded.delete(sedeId);
		} else {
			newExpanded.add(sedeId);
		}
		expandedSedes = newExpanded;
	}

	function getStatusColor(status: string) {
		switch (status) {
			case 'active':
				return 'bg-green-500';
			case 'offline':
				return 'bg-red-500';
			default:
				return 'bg-yellow-500';
		}
	}
</script>

<Card class="h-full">
	<CardHeader class="pb-3">
		<CardTitle class="text-base">Jerarquía de Red</CardTitle>
	</CardHeader>
	<CardContent class="p-0">
		{#if loading}
			<div class="space-y-2 p-4">
				{#each Array(5) as _}
					<div class="h-10 animate-pulse rounded bg-muted" />
				{/each}
			</div>
		{:else if sedes.length === 0}
			<p class="p-4 text-center text-sm text-muted-foreground">No hay sedes registradas</p>
		{:else}
			<ul class="px-2 pb-4">
				{#each sedes as sede}
					{@const isExpanded = expandedSedes.has(sede.id)}
					<li>
						<button
							type="button"
							class="flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left hover:bg-accent {selectedNodeId === sede.id ? 'bg-accent' : ''}"
							onclick={() => {
								toggleSede(sede.id);
								onSelectNode?.(sede.id, 'sede');
							}}
						>
							{#if isExpanded}
								<ChevronDown class="h-4 w-4 text-muted-foreground" />
							{:else}
								<ChevronRight class="h-4 w-4 text-muted-foreground" />
							{/if}
							<Building2 class="h-4 w-4 text-primary" />
							<span class="flex-1 text-sm font-medium">{sede.nombre}</span>
							<Badge variant="secondary" class="text-xs">
								{sede.device_count ?? sede.devices?.length ?? 0}
							</Badge>
						</button>

						{#if isExpanded && sede.devices && sede.devices.length > 0}
							<ul class="ml-6 mt-1 space-y-1 border-l border-muted pl-2">
								{#each sede.devices as device}
									<li>
										<button
											type="button"
											class="flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left hover:bg-accent {selectedNodeId === device.id ? 'bg-accent' : ''}"
											onclick={() => onSelectNode?.(device.id, 'device')}
										>
											<div class={cn('h-2 w-2 rounded-full', getStatusColor(device.status))} />
											<Monitor class="h-3 w-3 text-muted-foreground" />
											<span class="flex-1 text-sm">{device.hostname}</span>
										</button>
									</li>
								{/each}
							</ul>
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	</CardContent>
</Card>
