<!-- Ubicación: `apps/web/src/lib/components/devices/DeviceTable.svelte` -->
<!-- Descripción: Tabla de dispositivos con búsqueda, filtros y acciones -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { MoreHorizontal, Eye, Edit, Trash2 } from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuTrigger
	} from '$lib/components/ui/dropdown-menu';
	import { createQuery } from '@tanstack/svelte-query';
	import type { DeviceResponse } from '$lib/generated/api-types';

	interface Props {
		onViewDevice?: (id: string) => void;
		onEditDevice?: (id: string) => void;
		onDeleteDevice?: (id: string) => void;
	}

	let { onViewDevice, onEditDevice, onDeleteDevice }: Props = $props();

	let globalFilter = $state('');

	let query = createQuery({
		queryKey: ['devices'],
		queryFn: async (): Promise<DeviceResponse[]> => {
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/devices`);
			if (!response.ok) throw new Error('Failed to fetch devices');
			return response.json();
		},
		staleTime: 60 * 1000,
	});

	let filteredDevices = $derived(
		($query.data ?? []).filter(
			(device) =>
				device.hostname.toLowerCase().includes(globalFilter.toLowerCase()) ||
				device.ip_address.includes(globalFilter) ||
				device.device_type.toLowerCase().includes(globalFilter.toLowerCase())
		)
	);

	function getStatusBadge(status: string) {
		switch (status) {
			case 'active':
				return { label: 'Activo', class: 'bg-green-500/10 text-green-500 border-green-500/20' };
			case 'offline':
				return { label: 'Offline', class: 'bg-red-500/10 text-red-500 border-red-500/20' };
			default:
				return { label: 'Mantenimiento', class: 'bg-blue-500/10 text-blue-500 border-blue-500/20' };
		}
	}
</script>

<div class="space-y-4">
	<div class="flex items-center gap-4">
		<Input
			placeholder="Buscar por hostname, IP o tipo..."
			bind:value={globalFilter}
			class="max-w-sm"
		/>
	</div>

	{#if $query.isLoading}
		<div class="rounded-md border">
			{#each Array(5) as _, i}
				<div class="flex items-center gap-4 border-b p-4 last:border-0">
					<div class="h-4 w-24 animate-pulse rounded bg-muted" />
					<div class="h-4 w-32 animate-pulse rounded bg-muted" />
					<div class="h-4 w-20 animate-pulse rounded bg-muted" />
				</div>
			{/each}
		</div>
	{:else}
		<div class="rounded-md border">
			<table class="w-full">
				<thead>
					<tr class="border-b bg-muted/50">
						<th class="p-3 text-left text-sm font-medium">Hostname</th>
						<th class="p-3 text-left text-sm font-medium">IP</th>
						<th class="p-3 text-left text-sm font-medium">Tipo</th>
						<th class="p-3 text-left text-sm font-medium">Estado</th>
						<th class="p-3 text-left text-sm font-medium">Última conexión</th>
						<th class="p-3 text-left text-sm font-medium w-12">Acciones</th>
					</tr>
				</thead>
				<tbody>
					{#each filteredDevices as device (device.id)}
						{@const status = getStatusBadge(device.status)}
						<tr class="border-b transition-colors hover:bg-muted/50">
							<td class="p-3 font-medium">{device.hostname}</td>
							<td class="p-3"><code class="text-sm">{device.ip_address}</code></td>
							<td class="p-3 capitalize">{device.device_type}</td>
							<td class="p-3">
								<Badge class="text-xs {status.class}">{status.label}</Badge>
							</td>
							<td class="p-3 text-sm text-muted-foreground">
								{device.last_seen_at
									? new Date(device.last_seen_at).toLocaleString('es-BO')
									: 'Nunca'}
							</td>
							<td class="p-3">
								<DropdownMenu>
									<DropdownMenuTrigger>
										<Button variant="ghost" class="h-8 w-8 p-0">
											<MoreHorizontal class="h-4 w-4" />
										</Button>
									</DropdownMenuTrigger>
									<DropdownMenuContent align="end">
										<DropdownMenuItem onSelect={() => onViewDevice?.(device.id)}>
											<Eye class="mr-2 h-4 w-4" />
											Ver detalles
										</DropdownMenuItem>
										<DropdownMenuItem onSelect={() => onEditDevice?.(device.id)}>
											<Edit class="mr-2 h-4 w-4" />
											Editar
										</DropdownMenuItem>
										<DropdownMenuItem
											class="text-red-500 focus:text-red-500"
											onSelect={() => onDeleteDevice?.(device.id)}
										>
											<Trash2 class="mr-2 h-4 w-4" />
											Eliminar
										</DropdownMenuItem>
									</DropdownMenuContent>
								</DropdownMenu>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		{#if filteredDevices.length === 0}
			<p class="py-8 text-center text-muted-foreground">No se encontraron dispositivos</p>
		{/if}
	{/if}
</div>
