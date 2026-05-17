<!-- Ubicación: `apps/web/src/lib/components/devices/DeviceForm.svelte` -->
<!-- Descripción: Modal formulario para crear/editar dispositivos con validación Zod -->
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
	import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '$lib/components/ui/select';
	import { DeviceFormSchema } from '$lib/validation/schemas';
	import { createMutation } from '@tanstack/svelte-query';
	import { toast } from 'svelte-sonner';
	import { createEventDispatcher } from 'svelte';

	interface Props {
		open?: boolean;
		device?: {
			id?: string;
			hostname: string;
			ip_address: string;
			device_type: string;
			sede_id: string;
			mac_address?: string;
		};
		sedes?: { id: string; nombre: string }[];
		onClose?: () => void;
	}

	let { open = false, device, sedes = [], onClose }: Props = $props();

	const dispatch = createEventDispatcher();

	let hostname = $state(device?.hostname ?? '');
	let ip_address = $state(device?.ip_address ?? '');
	let device_type = $state(device?.device_type ?? '');
	let mac_address = $state(device?.mac_address ?? '');
	let sede_id = $state(device?.sede_id ?? '');
	let errors = $state<Record<string, string>>({});
	let loading = $state(false);

	let isEditing = $derived(!!device?.id);

	let mutation = createMutation({
		mutationFn: async (data: typeof DeviceFormSchema.infer) => {
			const url = isEditing
				? `${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/devices/${device!.id}`
				: `${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/devices`;
			const method = isEditing ? 'PUT' : 'POST';
			const response = await fetch(url, {
				method,
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(data),
			});
			if (!response.ok) throw new Error('Failed to save device');
			return response.json();
		},
		onSuccess: () => {
			toast.success(isEditing ? 'Dispositivo actualizado' : 'Dispositivo creado');
			dispatch('save');
			handleClose();
		},
		onError: (error) => {
			toast.error(error.message || 'Error al guardar dispositivo');
			loading = false;
		},
	});

	function validate(): boolean {
		const result = DeviceFormSchema.safeParse({
			hostname,
			ip_address,
			device_type,
			sede_id,
		});

		if (!result.success) {
			const fieldErrors: Record<string, string> = {};
			for (const issue of result.error.issues) {
				const path = issue.path[0] as string;
				fieldErrors[path] = issue.message;
			}
			errors = fieldErrors;
			return false;
		}

		errors = {};
		return true;
	}

	function handleSubmit() {
		if (!validate()) return;
		loading = true;
		mutation.mutate({
			hostname,
			ip_address,
			device_type,
			sede_id,
		});
	}

	function handleClose() {
		hostname = '';
		ip_address = '';
		device_type = '';
		mac_address = '';
		sede_id = '';
		errors = {};
		onClose?.();
	}
</script>

<Dialog {open} onOpenChange={(o) => !o && handleClose()}>
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
					<Input
						id="hostname"
						bind:value={hostname}
						placeholder="sw-central-01"
						class={errors.hostname ? 'border-destructive' : ''}
					/>
					{#if errors.hostname}
						<p class="text-sm text-destructive">{errors.hostname}</p>
					{/if}
				</div>

				<div class="grid gap-2">
					<Label for="ip_address">Dirección IP</Label>
					<Input
						id="ip_address"
						type="text"
						bind:value={ip_address}
						placeholder="192.168.1.1"
						class={errors.ip_address ? 'border-destructive' : ''}
					/>
					{#if errors.ip_address}
						<p class="text-sm text-destructive">{errors.ip_address}</p>
					{/if}
				</div>

				<div class="grid gap-2">
					<Label for="mac_address">Dirección MAC (opcional)</Label>
					<Input
						id="mac_address"
						bind:value={mac_address}
						placeholder="AA:BB:CC:DD:EE:FF"
					/>
				</div>

				<div class="grid gap-2">
					<Label for="device_type">Tipo de dispositivo</Label>
					<Select bind:value={device_type}>
						<SelectTrigger class={errors.device_type ? 'border-destructive' : ''}>
							<SelectValue placeholder="Seleccionar tipo" />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="switch">Switch</SelectItem>
							<SelectItem value="router">Router</SelectItem>
							<SelectItem value="firewall">Firewall</SelectItem>
							<SelectItem value="server">Servidor</SelectItem>
							<SelectItem value="workstation">Workstation</SelectItem>
							<SelectItem value="access_point">Access Point</SelectItem>
							<SelectItem value="printer">Impresora</SelectItem>
							<SelectItem value="other">Otro</SelectItem>
						</SelectContent>
					</Select>
					{#if errors.device_type}
						<p class="text-sm text-destructive">{errors.device_type}</p>
					{/if}
				</div>

				<div class="grid gap-2">
					<Label for="sede">Sede</Label>
					<Select bind:value={sede_id}>
						<SelectTrigger class={errors.sede_id ? 'border-destructive' : ''}>
							<SelectValue placeholder="Seleccionar sede" />
						</SelectTrigger>
						<SelectContent>
							{#each sedes as sede}
								<SelectItem value={sede.id}>{sede.nombre}</SelectItem>
							{/each}
						</SelectContent>
					</Select>
					{#if errors.sede_id}
						<p class="text-sm text-destructive">{errors.sede_id}</p>
					{/if}
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
