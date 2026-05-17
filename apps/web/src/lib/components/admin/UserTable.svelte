<!-- Ubicación: `apps/web/src/lib/components/admin/UserTable.svelte` -->
<!-- Descripción: Tabla de usuarios con acciones de crear/editar/eliminar -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0006 (RBAC) -->
<script lang="ts">
	import { MoreHorizontal, User, Shield, Trash2 } from 'lucide-svelte';
	import { Card, CardContent } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuSeparator,
		DropdownMenuTrigger
	} from '$lib/components/ui/dropdown-menu';
	import type { User } from '$lib/generated/api-types';

	interface Props {
		users?: User[];
		loading?: boolean;
		onEdit?: (userId: string) => void;
		onDelete?: (userId: string) => void;
	}

	let { users = [], loading = false, onEdit, onDelete }: Props = $props();

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
				<User class="h-12 w-12 text-muted-foreground mb-4" />
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
							<th class="p-3 text-left text-sm font-medium w-12">Acciones</th>
						</tr>
					</thead>
					<tbody>
						{#each filteredUsers as user (user.id)}
							<tr class="border-b transition-colors hover:bg-muted/50">
								<td class="p-3">
									<div class="flex items-center gap-3">
										<div class="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10">
											<User class="h-4 w-4 text-primary" />
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
									<DropdownMenu>
										<DropdownMenuTrigger>
											<Button variant="ghost" class="h-8 w-8 p-0">
												<MoreHorizontal class="h-4 w-4" />
											</Button>
										</DropdownMenuTrigger>
										<DropdownMenuContent align="end">
											<DropdownMenuItem onSelect={() => onEdit?.(user.id)}>
												<User class="mr-2 h-4 w-4" />
												Editar usuario
											</DropdownMenuItem>
											<DropdownMenuSeparator />
											<DropdownMenuItem
												class="text-red-500 focus:text-red-500"
												onSelect={() => onDelete?.(user.id)}
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
			</CardContent>
		</Card>
	{/if}
</div>
