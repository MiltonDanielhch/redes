<!-- Ubicación: `apps/web/src/routes/(dashboard)/sedes/+page.svelte` -->
<!-- Descripción: Página de gestión de sedes regionales con lista y modal de creación -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { Plus, Building2, MapPin, Server } from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import {
		Dialog,
		DialogContent,
		DialogDescription,
		DialogFooter,
		DialogHeader,
		DialogTitle,
	} from '$lib/components/ui/dialog';
	import { createQuery } from '@tanstack/svelte-query';
	import { toast } from 'svelte-sonner';
	import type { SedeResponse } from '$lib/generated/api-types';

	let showCreateModal = $state(false);
	let searchQuery = $state('');

	let nombre = $state('');
	let ubicacion = $state('');
	let secretaria = $state('');
	let loading = $state(false);

	let sedesQuery = createQuery({
		queryKey: ['sedes'],
		queryFn: async (): Promise<SedeResponse[]> => {
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/sedes`);
			if (!response.ok) throw new Error('Failed to fetch sedes');
			return response.json();
		},
		staleTime: 60 * 1000,
	});

	let filteredSedes = $derived(
		($sedesQuery.data ?? []).filter(
			(sede) =>
				sede.nombre.toLowerCase().includes(searchQuery.toLowerCase()) ||
				sede.ubicacion.toLowerCase().includes(searchQuery.toLowerCase()) ||
				sede.secretaria.toLowerCase().includes(searchQuery.toLowerCase())
		)
	);

	async function handleCreateSede() {
		if (!nombre || !ubicacion || !secretaria) {
			toast.error('Por favor completa todos los campos');
			return;
		}

		loading = true;
		try {
			const response = await fetch(
				`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/sedes`,
				{
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({ nombre, ubicacion, secretaria }),
				}
			);

			if (!response.ok) throw new Error('Failed to create sede');

			toast.success('Sede creada exitosamente');
			handleCloseModal();
			window.location.reload();
		} catch {
			toast.error('Error al crear sede');
		} finally {
			loading = false;
		}
	}

	function handleCloseModal() {
		showCreateModal = false;
		nombre = '';
		ubicacion = '';
		secretaria = '';
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold tracking-tight">Sedes Regionales</h1>
			<p class="text-muted-foreground">Gestión de sedes de la Gobernación del Beni</p>
		</div>
		<Button onclick={() => (showCreateModal = true)}>
			<Plus class="mr-2 h-4 w-4" />
			Nueva Sede
		</Button>
	</div>

	<div class="flex items-center gap-4">
		<Input
			placeholder="Buscar por nombre, ubicación o secretaría..."
			bind:value={searchQuery}
			class="max-w-sm"
		/>
	</div>

	{#if $sedesQuery.isLoading}
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each Array(6) as _}
				<Card>
					<CardHeader>
						<div class="h-6 w-3/4 animate-pulse rounded bg-muted" />
					</CardHeader>
					<CardContent>
						<div class="h-4 w-full animate-pulse rounded bg-muted" />
					</CardContent>
				</Card>
			{/each}
		</div>
	{:else if filteredSedes.length === 0}
		<div class="flex flex-col items-center justify-center py-12 text-center">
			<Building2 class="h-12 w-12 text-muted-foreground mb-4" />
			<p class="text-lg font-medium">No se encontraron sedes</p>
			<p class="text-muted-foreground">Crea una nueva sede para comenzar</p>
		</div>
	{:else}
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each filteredSedes as sede}
				<a href="/dashboard/sedes/{sede.id}" class="group">
					<Card class="h-full transition-colors hover:border-primary/50">
						<CardHeader class="flex flex-row items-start justify-between space-y-0">
							<div class="space-y-1">
								<CardTitle class="text-lg">{sede.nombre}</CardTitle>
								<CardDescription class="flex items-center gap-1">
									<MapPin class="h-3 w-3" />
									{sede.ubicacion}
								</CardDescription>
							</div>
							<Badge variant="secondary" class="ml-2">
								<Server class="h-3 w-3 mr-1" />
								--
							</Badge>
						</CardHeader>
						<CardContent>
							<div class="space-y-2">
								<div class="flex items-center justify-between text-sm">
									<span class="text-muted-foreground">Secretaría</span>
									<span class="font-medium">{sede.secretaria}</span>
								</div>
								<div class="flex items-center justify-between text-sm">
									<span class="text-muted-foreground">Dispositivos</span>
									<span class="font-medium">--</span>
								</div>
								<div class="flex items-center justify-between text-sm">
									<span class="text-muted-foreground">Estado</span>
									<Badge class="bg-green-500/10 text-green-500 border-green-500/20">
										Activo
									</Badge>
								</div>
							</div>
						</CardContent>
					</Card>
				</a>
			{/each}
		</div>
	{/if}
</div>

<Dialog open={showCreateModal} onOpenChange={(o) => !o && handleCloseModal()}>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<DialogTitle>Nueva Sede</DialogTitle>
			<DialogDescription>
				Ingresa la información de la nueva sede regional
			</DialogDescription>
		</DialogHeader>

		<form onsubmit={(e) => { e.preventDefault(); handleCreateSede(); }}>
			<div class="grid gap-4 py-4">
				<div class="grid gap-2">
					<Label for="nombre">Nombre</Label>
					<Input
						id="nombre"
						bind:value={nombre}
						placeholder="Riberalta"
					/>
				</div>
				<div class="grid gap-2">
					<Label for="ubicacion">Ubicación</Label>
					<Input
						id="ubicacion"
						bind:value={ubicacion}
						placeholder="Av. 6 de Agosto #123"
					/>
				</div>
				<div class="grid gap-2">
					<Label for="secretaria">Secretaría</Label>
					<Input
						id="secretaria"
						bind:value={secretaria}
						placeholder="Secretaría de Planificación"
					/>
				</div>
			</div>

			<DialogFooter>
				<Button type="button" variant="outline" onclick={handleCloseModal}>
					Cancelar
				</Button>
				<Button type="submit" disabled={loading}>
					{loading ? 'Creando...' : 'Crear Sede'}
				</Button>
			</DialogFooter>
		</form>
	</DialogContent>
</Dialog>
