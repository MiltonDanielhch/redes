<!-- Ubicación: `apps/web/src/routes/(dashboard)/dashboard/+page.svelte` -->
<!-- Descripción: Dashboard principal con resumen de métricas, alertas y estado de red -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { Building2, Monitor, AlertTriangle, TrendingUp, Clock, Activity } from 'lucide-svelte';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
</script>

<div class="space-y-6">
	<div>
		<h1 class="text-3xl font-bold tracking-tight">Dashboard</h1>
		<p class="text-muted-foreground">Resumen del monitoreo de infraestructura regional</p>
	</div>

	<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Sedes Activas</CardTitle>
				<Building2 class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">12</div>
				<p class="text-xs text-muted-foreground">+2 desde el mes pasado</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Dispositivos Online</CardTitle>
				<Monitor class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">847</div>
				<p class="text-xs text-muted-foreground">98.2% disponibilidad</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Alertas Pendientes</CardTitle>
				<AlertTriangle class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">23</div>
				<p class="text-xs text-muted-foreground">5 críticas</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Ancho de Banda</CardTitle>
				<TrendingUp class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">4.2 Gbps</div>
				<p class="text-xs text-muted-foreground">+12% promedio</p>
			</CardContent>
		</Card>
	</div>

	<div class="grid gap-4 md:grid-cols-2">
		<Card>
			<CardHeader>
				<CardTitle>Actividad Reciente</CardTitle>
				<CardDescription>Últimas alertas y eventos</CardDescription>
			</CardHeader>
			<CardContent>
				<div class="space-y-4">
					<div class="flex items-center gap-4">
						<div class="flex h-9 w-9 items-center justify-center rounded-full bg-destructive/10">
							<AlertTriangle class="h-4 w-4 text-destructive" />
						</div>
						<div class="flex-1 space-y-1">
							<p class="text-sm font-medium">Switch SW-CENTRAL-01 offline</p>
							<p class="text-xs text-muted-foreground">Hace 5 minutos</p>
						</div>
						<Badge variant="destructive">Crítica</Badge>
					</div>
					<div class="flex items-center gap-4">
						<div class="flex h-9 w-9 items-center justify-center rounded-full bg-yellow-500/10">
							<Clock class="h-4 w-4 text-yellow-500" />
						</div>
						<div class="flex-1 space-y-1">
							<p class="text-sm font-medium">Alta latencia en RBN-BO-001</p>
							<p class="text-xs text-muted-foreground">Hace 15 minutos</p>
						</div>
						<Badge variant="default">Media</Badge>
					</div>
					<div class="flex items-center gap-4">
						<div class="flex h-9 w-9 items-center justify-center rounded-full bg-green-500/10">
							<Activity class="h-4 w-4 text-green-500" />
						</div>
						<div class="flex-1 space-y-1">
							<p class="text-sm font-medium">Conexión restaurada</p>
							<p class="text-xs text-muted-foreground">Hace 1 hora</p>
						</div>
						<Badge variant="secondary">Info</Badge>
					</div>
				</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>Estado de la Red</CardTitle>
				<CardDescription>Métricas en tiempo real</CardDescription>
			</CardHeader>
			<CardContent>
				<div class="space-y-4">
					<div class="flex items-center justify-between">
						<span class="text-sm">CPU Promedio</span>
						<span class="text-sm font-medium">45%</span>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-sm">Memoria Usada</span>
						<span class="text-sm font-medium">62%</span>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-sm">Paquetes Perdidos</span>
						<span class="text-sm font-medium text-green-500">0.1%</span>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-sm">Uptime Promedio</span>
						<span class="text-sm font-medium">99.7%</span>
					</div>
				</div>
			</CardContent>
		</Card>
	</div>
</div>
