<!-- Ubicación: `apps/web/src/lib/components/layout/Topbar.svelte` -->
<!-- Descripción: Barra superior con búsqueda, notificaciones, avatar y dropdown de usuario -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit) -->
<script lang="ts">
	import { Bell, Search, User, LogOut, Menu } from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import { Badge } from '$lib/components/ui/badge';
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuLabel,
		DropdownMenuSeparator,
		DropdownMenuTrigger
	} from '$lib/components/ui/dropdown-menu';
	import { auth } from '$lib/stores/auth.svelte';

	let { user }: { user?: { nombre: string | null; email: string | null } | null } = $props();
</script>

<header
	class="sticky top-0 z-40 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60"
>
	<div class="flex h-14 items-center gap-4 px-4">
		<Button variant="ghost" size="icon" class="lg:hidden">
			<Menu class="h-5 w-5" />
		</Button>

		<div class="flex-1">
			<div class="relative hidden max-w-md md:block">
				<Search class="absolute left-2.5 top-2 h-4 w-4 text-muted-foreground" />
				<Input
					type="search"
					placeholder="Buscar..."
					class="pl-8"
				/>
			</div>
		</div>

		<div class="flex items-center gap-2">
			<Button variant="ghost" size="icon" class="relative">
				<Bell class="h-5 w-5" />
				<Badge
					class="absolute -right-1 -top-1 h-5 w-5 rounded-full p-0 text-[10px]"
					variant="destructive"
				>
					3
				</Badge>
			</Button>

			<DropdownMenu>
				<DropdownMenuTrigger>
					<Avatar class="h-8 w-8">
						<AvatarImage src="" />
						<AvatarFallback>
							<User class="h-4 w-4" />
						</AvatarFallback>
					</Avatar>
				</DropdownMenuTrigger>
				<DropdownMenuContent align="end" class="w-56">
					<DropdownMenuLabel>
						<div class="flex flex-col space-y-1">
							<p class="text-sm font-medium leading-none">
								{user?.nombre || 'Usuario'}
							</p>
							<p class="text-xs leading-none text-muted-foreground">
								{user?.email || 'email@ejemplo.com'}
							</p>
						</div>
					</DropdownMenuLabel>
					<DropdownMenuSeparator />
					<DropdownMenuItem>
						<User class="mr-2 h-4 w-4" />
						Perfil
					</DropdownMenuItem>
					<DropdownMenuSeparator />
					<DropdownMenuItem onclick={() => auth.clearAuth()}>
						<LogOut class="mr-2 h-4 w-4" />
						Cerrar sesión
					</DropdownMenuItem>
				</DropdownMenuContent>
			</DropdownMenu>
		</div>
	</div>
</header>
