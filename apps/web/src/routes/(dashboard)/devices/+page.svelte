<!-- Ubicación: `apps/web/src/routes/(dashboard)/devices/+page.svelte` -->
<!-- Descripción: Página de inventario de dispositivos con tabla, filtros y CRUD -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { Plus } from 'lucide-svelte';
	import DeviceTable from '$lib/components/devices/DeviceTable.svelte';
	import DeviceForm from '$lib/components/devices/DeviceForm.svelte';
	import { Button } from '$lib/components/ui/button';
	import { createQuery } from '@tanstack/svelte-query';

	let showForm = $state(false);
	let editingDeviceId = $state<string | null>(null);

	let sedesQuery = createQuery({
		queryKey: ['sedes'],
		queryFn: async () => {
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/sedes`);
			if (!response.ok) throw new Error('Failed to fetch sedes');
			return response.json();
		},
	});

	function handleViewDevice(id: string) {
		window.location.href = `/dashboard/devices/${id}`;
	}

	function handleEditDevice(id: string) {
		editingDeviceId = id;
		showForm = true;
	}

	async function handleDeleteDevice(id: string) {
		if (confirm('¿Estás seguro de eliminar este dispositivo?')) {
			try {
				await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/devices/${id}`, {
					method: 'DELETE',
				});
				window.location.reload();
			} catch {
				alert('Error al eliminar dispositivo');
			}
		}
	}

	function handleCloseForm() {
		showForm = false;
		editingDeviceId = null;
	}

	function handleDeviceSaved() {
		window.location.reload();
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold tracking-tight">Dispositivos</h1>
			<p class="text-muted-foreground">Inventario de dispositivos de red</p>
		</div>
		<Button onclick={() => (showForm = true)}>
			<Plus class="mr-2 h-4 w-4" />
			Nuevo Dispositivo
		</Button>
	</div>

	<DeviceTable
		onViewDevice={handleViewDevice}
		onEditDevice={handleEditDevice}
		onDeleteDevice={handleDeleteDevice}
	/>

	<DeviceForm
		open={showForm}
		device={editingDeviceId ? { id: editingDeviceId, hostname: '', ip_address: '', device_type: '', sede_id: '' } : undefined}
		sedes={$sedesQuery.data ?? []}
		onClose={handleCloseForm}
	/>
</div>
