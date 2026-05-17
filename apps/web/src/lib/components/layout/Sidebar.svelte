<!-- Ubicación: `apps/web/src/lib/components/layout/Sidebar.svelte` -->
<!-- Descripción: Sidebar de navegación principal con items de menú colapsables y control de permisos -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0006 (RBAC) -->
<script lang="ts">
	import {
		Home,
		Building2,
		Monitor,
		BarChart3,
		Network,
		AlertTriangle,
		ShieldAlert,
		Server,
		ScrollText,
		Users,
		Settings,
		ChevronLeft,
		ChevronRight
	} from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { cn } from '$lib/utils';
	import { auth } from '$lib/stores/auth.svelte';

	let collapsed = $state(false);

	const navItems = [
		{ href: '/dashboard', label: 'Inicio', icon: Home },
		{ href: '/dashboard/sedes', label: 'Sedes', icon: Building2 },
		{ href: '/dashboard/devices', label: 'Dispositivos', icon: Monitor },
		{ href: '/dashboard/metrics', label: 'Métricas', icon: BarChart3 },
		{ href: '/dashboard/topology', label: 'Topología', icon: Network },
		{ href: '/dashboard/alerts', label: 'Alertas', icon: AlertTriangle, badge: 5 },
		{ href: '/dashboard/intrusions', label: 'Intrusiones', icon: ShieldAlert },
		{ href: '/dashboard/agents', label: 'Agentes', icon: Server },
	];

	const adminItems = [
		{ href: '/dashboard/audit', label: 'Auditoría', icon: ScrollText, permission: 'audit:read' },
		{ href: '/dashboard/admin/users', label: 'Admin', icon: Users, permission: 'user:read' },
	];

	const bottomItems = [
		{ href: '/dashboard/settings', label: 'Configuración', icon: Settings },
	];

	function canAccess(permission?: string): boolean {
		if (!permission) return true;
		return auth.hasPermission(permission);
	}
</script>

<aside
	class={cn(
		'flex flex-col border-r bg-sidebar transition-all duration-300',
		collapsed ? 'w-16' : 'w-64'
	)}
>
	<div class="flex h-14 items-center border-b px-4">
		{#if !collapsed}
			<div class="flex items-center gap-2">
				<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
					<ShieldAlert class="h-5 w-5" />
				</div>
				<div class="flex flex-col">
					<span class="text-sm font-semibold">Redes Beni</span>
					<span class="text-xs text-muted-foreground">Monitoreo</span>
				</div>
			</div>
		{:else}
			<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
				<ShieldAlert class="h-5 w-5" />
			</div>
		{/if}
	</div>

	<nav class="flex-1 overflow-y-auto p-2">
		<ul class="space-y-1">
			{#each navItems as item}
				<li>
					<a
						href={item.href}
						class={cn(
							'flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors hover:bg-accent hover:text-accent-foreground',
							collapsed && 'justify-center px-2'
						)}
						title={collapsed ? item.label : undefined}
					>
						<item.icon class="h-5 w-5 shrink-0" />
						{#if !collapsed}
							<span class="flex-1">{item.label}</span>
							{#if item.badge}
								<Badge variant="destructive" class="h-5 min-w-5 justify-center px-1 text-xs">
									{item.badge}
								</Badge>
							{/if}
						{/if}
					</a>
				</li>
			{/each}
		</ul>

		{#if adminItems.some((item) => canAccess(item.permission))}
			<div class="my-4 border-t" />
			<ul class="space-y-1">
				{#each adminItems.filter((item) => canAccess(item.permission)) as item}
					<li>
						<a
							href={item.href}
							class={cn(
								'flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors hover:bg-accent hover:text-accent-foreground',
								collapsed && 'justify-center px-2'
							)}
							title={collapsed ? item.label : undefined}
						>
							<item.icon class="h-5 w-5 shrink-0" />
							{#if !collapsed}
								<span>{item.label}</span>
							{/if}
						</a>
					</li>
				{/each}
			</ul>
		{/if}
	</nav>

	<div class="border-t p-2">
		<ul class="space-y-1">
			{#each bottomItems as item}
				<li>
					<a
						href={item.href}
						class={cn(
							'flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors hover:bg-accent hover:text-accent-foreground',
							collapsed && 'justify-center px-2'
						)}
						title={collapsed ? item.label : undefined}
					>
						<item.icon class="h-5 w-5 shrink-0" />
						{#if !collapsed}
							<span>{item.label}</span>
						{/if}
					</a>
				</li>
			{/each}
		</ul>

		<div class="mt-2 pt-2">
			<Button
				variant="ghost"
				size="sm"
				class={cn('w-full', collapsed ? 'px-2' : 'justify-start')}
				onclick={() => (collapsed = !collapsed)}
			>
				{#if collapsed}
					<ChevronRight class="h-4 w-4" />
				{:else}
					<ChevronLeft class="mr-2 h-4 w-4" />
					Collapse
				{/if}
			</Button>
		</div>
	</div>
</aside>
