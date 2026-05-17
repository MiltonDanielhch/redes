<!-- Ubicación: `apps/web/src/lib/components/devices/DeviceForm.svelte` -->
<!-- Descripción: Modal formulario para crear/editar dispositivos -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import {
		Dialog,
		DialogContent,
		DialogDescription,
		DialogFooter,
		DialogHeader,
		DialogTitle
	} from '$lib/components/ui/dialog';
	import { toast } from 'svelte-sonner';

	interface Props {
		open?: boolean;
		device?: {
			id?: string;
			hostname: string;
			ip_address: string;
			device_type: string;
			sede_id: string;
		};
		sedes?: { id: string; nombre: string }[];
		onClose?: () => void;
		onSave?: () => void;
	}

	let { open = false, device, sedes = [], onClose, onSave }: Props = $props();

	let hostname = $state('');
	let ip_address = $state('');
	let device_type = $state('');
	let sede_id = $state('');
	let loading = $state(false);

	let isEditing = $derived(!!device?.id);

	function validate(): boolean {
		if (!hostname || !ip_address || !device_type || !sede_id) {
			toast.error('Por favor completa todos los campos');
			return false;
		}
		return true;
	}

	async function handleSubmit() {
		if (!validate()) return;

		loading = true;
		try {
			const url = isEditing
				? `${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/devices/${device!.id}`
				: `${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/devices`;

			const response = await fetch(url, {
				method: isEditing ? 'PUT' : 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ hostname, ip_address, device_type, sede_id }),
			});

			if (!response.ok) throw new Error('Failed to save');

			toast.success(isEditing ? 'Dispositivo actualizado' : 'Dispositivo creado');
			onSave?.();
			handleClose();
		} catch {
			toast.error('Error al guardar dispositivo');
		} finally {
			loading = false;
		}
	}

	function handleClose() {
		hostname = '';
		ip_address = '';
		device_type = '';
		sede_id = '';
		onClose?.();
	}
</script>

<Dialog open={open} onOpenChange={(o) => !o && handleClose()}>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<DialogTitle>{isEditing ? 'Editar Dispositivo' : 'Nuevo Dispositivo'}</DialogTitle>
			<DialogDescription>
				{isEditing ? 'Actualiza la información del dispositivo' : 'Ingresa la información del nuevo dispositivo'}
			</DialogDescription>
		</DialogHeader>

		<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
			<div class="grid gap-4 py-4">
				<div class="grid gap-2">
					<Label for="hostname">Hostname</Label>
					<Input id="hostname" bind:value={hostname} placeholder="sw-central-01" />
				</div>

				<div class="grid gap-2">
					<Label for="ip_address">Dirección IP</Label>
					<Input id="ip_address" bind:value={ip_address} placeholder="192.168.1.1" />
				</div>

				<div class="grid gap-2">
					<Label for="device_type">Tipo</Label>
					<Input id="device_type" bind:value={device_type} placeholder="switch, router, etc." />
				</div>

				<div class="grid gap-2">
					<Label for="sede">Sede</Label>
					<select id="sede" bind:value={sede_id} class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm">
						<option value="">Seleccionar sede</option>
						{#each sedes as sede}
							<option value={sede.id}>{sede.nombre}</option>
						{/each}
					</select>
				</div>
			</div>

			<DialogFooter>
				<Button type="button" variant="outline" onclick={handleClose}>
					Cancelar
				</Button>
				<Button type="submit" disabled={loading}>
					{loading ? 'Guardando...' : isEditing ? 'Actualizar' : 'Crear'}
				</Button>
			</DialogFooter>
		</form>
	</DialogContent>
</Dialog>
