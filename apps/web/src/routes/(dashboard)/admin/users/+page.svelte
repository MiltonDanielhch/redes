<!-- Ubicación: `apps/web/src/routes/(dashboard)/admin/users/+page.svelte` -->
<!-- Descripción: Página de administración de usuarios con tabla y modal de creación/edición -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0006 (RBAC) -->
<script lang="ts">
	import UserTable from '$lib/components/admin/UserTable.svelte';
	import { createQuery, useQueryClient } from '@tanstack/svelte-query';
	import { toast } from 'svelte-sonner';
	import { Plus } from 'lucide-svelte';
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
	import type { User } from '$lib/generated/api-types';

	let showModal = $state(false);
	let editingUser = $state<User | null>(null);

	let email = $state('');
	let password = $state('');
	let nombre = $state('');
	let role = $state('viewer');
	let loading = $state(false);

	const queryClient = useQueryClient();

	let usersQuery = createQuery({
		queryKey: ['users'],
		queryFn: async (): Promise<User[]> => {
			const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/users`);
			if (!response.ok) throw new Error('Failed to fetch users');
			return response.json();
		},
		staleTime: 60 * 1000,
	});

	function openCreateModal() {
		editingUser = null;
		email = '';
		password = '';
		nombre = '';
		role = 'viewer';
		showModal = true;
	}

	function openEditModal(user: User) {
		editingUser = user;
		email = user.email;
		password = '';
		nombre = user.nombre;
		role = user.roles[0] || 'viewer';
		showModal = true;
	}

	async function handleSubmit() {
		if (!email || !nombre) {
			toast.error('Por favor completa los campos requeridos');
			return;
		}

		loading = true;
		try {
			const url = editingUser
				? `${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/users/${editingUser.id}`
				: `${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/users`;

			const body: Record<string, string> = { email, nombre, role };
			if (password) body.password = password;

			const response = await fetch(url, {
				method: editingUser ? 'PUT' : 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(body),
			});

			if (!response.ok) throw new Error('Failed to save user');

			toast.success(editingUser ? 'Usuario actualizado' : 'Usuario creado');
			showModal = false;
			queryClient.invalidateQueries({ queryKey: ['users'] });
		} catch {
			toast.error('Error al guardar usuario');
		} finally {
			loading = false;
		}
	}

	async function handleDelete(userId: string) {
		if (!confirm('¿Estás seguro de eliminar este usuario?')) return;

		try {
			const response = await fetch(
				`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/users/${userId}`,
				{ method: 'DELETE' }
			);
			if (!response.ok) throw new Error('Failed to delete user');
			toast.success('Usuario eliminado');
			queryClient.invalidateQueries({ queryKey: ['users'] });
		} catch {
			toast.error('Error al eliminar usuario');
		}
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold tracking-tight">Administración de Usuarios</h1>
			<p class="text-muted-foreground">Gestiona usuarios, roles y permisos del sistema</p>
		</div>
		<Button onclick={openCreateModal}>
			<Plus class="mr-2 h-4 w-4" />
			Nuevo Usuario
		</Button>
	</div>

	<UserTable
		users={$usersQuery.data}
		loading={$usersQuery.isLoading}
		onEdit={openEditModal}
		onDelete={handleDelete}
	/>
</div>

<Dialog open={showModal} onOpenChange={(o) => !o && (showModal = false)}>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<DialogTitle>{editingUser ? 'Editar Usuario' : 'Nuevo Usuario'}</DialogTitle>
			<DialogDescription>
				{editingUser ? 'Actualiza la información del usuario' : 'Ingresa la información del nuevo usuario'}
			</DialogDescription>
		</DialogHeader>

		<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
			<div class="grid gap-4 py-4">
				<div class="grid gap-2">
					<Label for="nombre">Nombre</Label>
					<Input id="nombre" bind:value={nombre} placeholder="Juan Pérez" />
				</div>

				<div class="grid gap-2">
					<Label for="email">Email</Label>
					<Input id="email" type="email" bind:value={email} placeholder="juan@ejemplo.com" />
				</div>

				<div class="grid gap-2">
					<Label for="password">{editingUser ? 'Nueva Contraseña (opcional)' : 'Contraseña'}</Label>
					<Input id="password" type="password" bind:value={password} placeholder="••••••••" />
				</div>

				<div class="grid gap-2">
					<Label for="role">Rol</Label>
					<select
						id="role"
						bind:value={role}
						class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
					>
						<option value="viewer">Viewer</option>
						<option value="operator">Operator</option>
						<option value="admin">Admin</option>
					</select>
				</div>
			</div>

			<DialogFooter>
				<Button type="button" variant="outline" onclick={() => (showModal = false)}>
					Cancelar
				</Button>
				<Button type="submit" disabled={loading}>
					{loading ? 'Guardando...' : editingUser ? 'Actualizar' : 'Crear'}
				</Button>
			</DialogFooter>
		</form>
	</DialogContent>
</Dialog>
