<!-- Ubicación: `apps/web/src/lib/components/topology/TopologyMap.svelte` -->
<!-- Descripción: Mapa de topología SVG interactivo con nodes (sedes/dispositivos) y edges (conexiones) -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';

	interface Node {
		id: string;
		label: string;
		type: 'sede' | 'device' | 'gateway';
		status: 'online' | 'offline' | 'warning';
		x: number;
		y: number;
	}

	interface Edge {
		source: string;
		target: string;
		bandwidth?: string;
		latency?: number;
	}

	interface Props {
		nodes?: Node[];
		edges?: Edge[];
		selectedNodeId?: string | null;
		onNodeClick?: (nodeId: string) => void;
		loading?: boolean;
	}

	let {
		nodes = [],
		edges = [],
		selectedNodeId = null,
		onNodeClick,
		loading = false
	}: Props = $props();

	function getNodeColor(status: string) {
		switch (status) {
			case 'online':
				return '#22c55e';
			case 'warning':
				return '#eab308';
			case 'offline':
				return '#ef4444';
			default:
				return '#6b7280';
		}
	}

	function getNodeIcon(type: string) {
		switch (type) {
			case 'sede':
				return '🏢';
			case 'gateway':
				return '🌐';
			default:
				return '🖥️';
		}
	}

	let viewBox = $derived(() => {
		if (nodes.length === 0) return '0 0 800 600';
		const minX = Math.min(...nodes.map((n) => n.x)) - 50;
		const maxX = Math.max(...nodes.map((n) => n.x)) + 50;
		const minY = Math.min(...nodes.map((n) => n.y)) - 50;
		const maxY = Math.max(...nodes.map((n) => n.y)) + 50;
		return `${minX} ${minY} ${maxX - minX} ${maxY - minY}`;
	});
</script>

<Card>
	<CardHeader>
		<CardTitle>Topología de Red</CardTitle>
	</CardHeader>
	<CardContent>
		{#if loading}
			<div class="h-96 animate-pulse rounded bg-muted" />
		{:else if nodes.length === 0}
			<div class="flex h-96 flex-col items-center justify-center text-muted-foreground">
				<p class="text-lg font-medium">Sin datos de topología</p>
				<p class="text-sm">Selecciona una sede para ver su topología</p>
			</div>
		{:else}
			<div class="relative h-96 w-full overflow-auto rounded-lg border bg-background">
				<svg class="h-full w-full" viewBox={viewBox()}>
					<defs>
						<marker
							id="arrowhead"
							markerWidth="10"
							markerHeight="7"
							refX="9"
							refY="3.5"
							orient="auto"
						>
							<polygon points="0 0, 10 3.5, 0 7" fill="#6b7280" />
						</marker>
					</defs>

					{#each edges as edge}
						{@const sourceNode = nodes.find((n) => n.id === edge.source)}
						{@const targetNode = nodes.find((n) => n.id === edge.target)}
						{#if sourceNode && targetNode}
							<line
								x1={sourceNode.x}
								y1={sourceNode.y}
								x2={targetNode.x}
								y2={targetNode.y}
								stroke="#6b7280"
								stroke-width="2"
								marker-end="url(#arrowhead)"
							/>
							{#if edge.bandwidth}
								<text
									x={(sourceNode.x + targetNode.x) / 2}
									y={(sourceNode.y + targetNode.y) / 2 - 10}
									class="text-xs fill-gray-500"
									text-anchor="middle"
								>
									{edge.bandwidth}
								</text>
							{/if}
						{/if}
					{/each}

					{#each nodes as node}
						<g
							transform="translate({node.x}, {node.y})"
							onclick={() => onNodeClick?.(node.id)}
							style="cursor: pointer"
							role="button"
							tabindex="0"
						>
							<circle
								r="25"
								fill={getNodeColor(node.status)}
								stroke={selectedNodeId === node.id ? '#3b82f6' : 'white'}
								stroke-width={selectedNodeId === node.id ? '3' : '2'}
							/>
							<text
								text-anchor="middle"
								dominant-baseline="middle"
								class="pointer-events-none text-xl"
							>
								{getNodeIcon(node.type)}
							</text>
							<text
								y="40"
								text-anchor="middle"
								class="text-xs fill-gray-700 font-medium"
							>
								{node.label}
							</text>
						</g>
					{/each}
				</svg>
			</div>

			<div class="mt-4 flex flex-wrap gap-4 text-sm">
				<div class="flex items-center gap-2">
					<div class="h-3 w-3 rounded-full bg-green-500" />
					<span>Online</span>
				</div>
				<div class="flex items-center gap-2">
					<div class="h-3 w-3 rounded-full bg-yellow-500" />
					<span>Warning</span>
				</div>
				<div class="flex items-center gap-2">
					<div class="h-3 w-3 rounded-full bg-red-500" />
					<span>Offline</span>
				</div>
				<div class="flex items-center gap-2">
					<span class="text-muted-foreground">🖥️ Dispositivo</span>
				</div>
				<div class="flex items-center gap-2">
					<span class="text-muted-foreground">🌐 Gateway</span>
				</div>
				<div class="flex items-center gap-2">
					<span class="text-muted-foreground">🏢 Sede</span>
				</div>
			</div>
		{/if}
	</CardContent>
</Card>
