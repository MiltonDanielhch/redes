<!-- Ubicación: `apps/web/src/lib/components/admin/UserTable.svelte` -->
<!-- Descripción: Tabla de usuarios con acciones de crear/editar/eliminar -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0006 (RBAC) -->
<script lang="ts">
	import { User as UserIcon, Shield, Trash2, Edit } from 'lucide-svelte';
	import { Card, CardContent } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import type { User } from '$lib/generated/api-types';

	interface Props {
		users?: User[];
		loading?: boolean;
		onEditUser?: (userId: string) => void;
		onDeleteUser?: (userId: string) => void;
	}

	let { users = [], loading = false, onEditUser, onDeleteUser }: Props = $props();

	let searchQuery = $state('');

	let filteredUsers = $derived(
		(users ?? []).filter(
			(user) =>
				user.email.toLowerCase().includes(searchQuery.toLowerCase()) ||
				user.nombre.toLowerCase().includes(searchQuery.toLowerCase())
		)
	);

	function getRoleBadgeColor(role: string) {
		switch (role.toLowerCase()) {
			case 'admin':
				return 'bg-red-500/10 text-red-500 border-red-500/20';
			case 'operator':
				return 'bg-blue-500/10 text-blue-500 border-blue-500/20';
			default:
				return 'bg-gray-500/10 text-gray-500 border-gray-500/20';
		}
	}
</script>

<div class="space-y-4">
	<div class="flex items-center gap-4">
		<Input
			placeholder="Buscar por email o nombre..."
			bind:value={searchQuery}
			class="max-w-sm"
		/>
	</div>

	{#if loading}
		<Card>
			<CardContent class="p-6">
				<div class="space-y-3">
					{#each Array(5) as _}
						<div class="h-16 animate-pulse rounded bg-muted" />
					{/each}
				</div>
			</CardContent>
		</Card>
	{:else if filteredUsers.length === 0}
		<Card>
			<CardContent class="flex flex-col items-center justify-center py-12">
				<UserIcon class="h-12 w-12 text-muted-foreground mb-4" />
				<p class="text-lg font-medium">No se encontraron usuarios</p>
			</CardContent>
		</Card>
	{:else}
		<Card>
			<CardContent class="p-0">
				<table class="w-full">
					<thead>
						<tr class="border-b bg-muted/50">
							<th class="p-3 text-left text-sm font-medium">Usuario</th>
							<th class="p-3 text-left text-sm font-medium">Email</th>
							<th class="p-3 text-left text-sm font-medium">Roles</th>
							<th class="p-3 text-left text-sm font-medium">Fecha de creación</th>
							<th class="p-3 text-left text-sm font-medium w-24">Acciones</th>
						</tr>
					</thead>
					<tbody>
						{#each filteredUsers as user (user.id)}
							<tr class="border-b transition-colors hover:bg-muted/50">
								<td class="p-3">
									<div class="flex items-center gap-3">
										<div class="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10">
											<UserIcon class="h-4 w-4 text-primary" />
										</div>
										<span class="font-medium">{user.nombre}</span>
									</div>
								</td>
								<td class="p-3 text-sm">{user.email}</td>
								<td class="p-3">
									<div class="flex flex-wrap gap-1">
										{#each user.roles as role}
											<Badge class={getRoleBadgeColor(role)} variant="outline">
												{#if role === 'admin'}
													<Shield class="mr-1 h-3 w-3" />
												{/if}
												{role}
											</Badge>
										{/each}
									</div>
								</td>
								<td class="p-3 text-sm text-muted-foreground">
									{new Date(user.created_at).toLocaleDateString('es-BO')}
								</td>
								<td class="p-3">
									<div class="flex gap-2">
										<Button variant="ghost" size="sm" onclick={() => onEditUser?.(user.id)}>
											<Edit class="h-4 w-4" />
										</Button>
										<Button variant="ghost" size="sm" class="text-red-500 hover:text-red-500" onclick={() => onDeleteUser?.(user.id)}>
											<Trash2 class="h-4 w-4" />
										</Button>
									</div>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</CardContent>
		</Card>
	{/if}
</div>
