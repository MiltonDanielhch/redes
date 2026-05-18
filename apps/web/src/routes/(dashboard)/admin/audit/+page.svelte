<!-- Ubicación: `apps/web/src/routes/(dashboard)/admin/audit/+page.svelte` -->
<!-- Descripción: Página de logs de auditoría con filtros y exportación -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0006 (RBAC) -->
<script lang="ts">
	import { FileText, Download, Filter } from 'lucide-svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { createQuery } from '@tanstack/svelte-query';
	import type { AuditLogEntry } from '$lib/generated/api-types';

	let userFilter = $state('');
	let actionFilter = $state('');
	let searchQuery = $state('');

	let auditQuery = createQuery({
		queryKey: ['audit', userFilter, actionFilter],
		queryFn: async (): Promise<AuditLogEntry[]> => {
			const params = new URLSearchParams();
			if (userFilter) params.set('user_id', userFilter);
			if (actionFilter) params.set('action', actionFilter);
			const response = await fetch(
				`${import.meta.env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/audit?${params}`
			);
			if (!response.ok) throw new Error('Failed to fetch audit logs');
			return response.json();
		},
		staleTime: 60 * 1000,
	});

	let filteredLogs = $derived(
		($auditQuery.data ?? []).filter((log) =>
			!searchQuery ||
			log.action.toLowerCase().includes(searchQuery.toLowerCase()) ||
			log.resource.toLowerCase().includes(searchQuery.toLowerCase()) ||
			log.ip_address.includes(searchQuery)
		)
	);

	function getActionColor(action: string) {
		if (action.includes('create') || action.includes('CREATE')) return 'bg-green-500/10 text-green-500';
		if (action.includes('delete') || action.includes('DELETE')) return 'bg-red-500/10 text-red-500';
		if (action.includes('update') || action.includes('UPDATE')) return 'bg-blue-500/10 text-blue-500';
		if (action.includes('login') || action.includes('LOGIN')) return 'bg-purple-500/10 text-purple-500';
		return 'bg-gray-500/10 text-gray-500';
	}

	function formatDate(dateStr: string) {
		return new Date(dateStr).toLocaleString('es-BO');
	}

	async function exportToCsv() {
		const logs = filteredLogs;
		const csv = [
			['ID', 'Usuario', 'Acción', 'Recurso', 'IP', 'Fecha'].join(','),
			...logs.map((log) =>
				[log.id, log.user_id, log.action, log.resource, log.ip_address, log.created_at].join(',')
			),
		].join('\n');

		const blob = new Blob([csv], { type: 'text/csv' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `audit-log-${new Date().toISOString().split('T')[0]}.csv`;
		a.click();
		URL.revokeObjectURL(url);
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold tracking-tight">Auditoría</h1>
			<p class="text-muted-foreground">Logs de actividad del sistema</p>
		</div>
		<Button variant="outline" onclick={exportToCsv}>
			<Download class="mr-2 h-4 w-4" />
			Exportar CSV
		</Button>
	</div>

	<div class="flex flex-wrap items-center gap-4">
		<div class="flex items-center gap-2">
			<Filter class="h-4 w-4 text-muted-foreground" />
			<Input
				placeholder="Buscar..."
				bind:value={searchQuery}
				class="max-w-xs"
			/>
		</div>

		<select
			bind:value={actionFilter}
			class="flex h-10 rounded-md border border-input bg-background px-3 py-2 text-sm"
		>
			<option value="">Todas las acciones</option>
			<option value="LOGIN">Login</option>
			<option value="LOGOUT">Logout</option>
			<option value="CREATE">Create</option>
			<option value="UPDATE">Update</option>
			<option value="DELETE">Delete</option>
		</select>
	</div>

	{#if $auditQuery.isLoading}
		<Card>
			<CardContent class="p-6">
				<div class="space-y-3">
					{#each Array(10) as _}
						<div class="h-12 animate-pulse rounded bg-muted" />
					{/each}
				</div>
			</CardContent>
		</Card>
	{:else if filteredLogs.length === 0}
		<Card>
			<CardContent class="flex flex-col items-center justify-center py-12">
				<FileText class="h-12 w-12 text-muted-foreground mb-4" />
				<p class="text-lg font-medium">Sin registros</p>
				<p class="text-muted-foreground">No hay logs de auditoría disponibles</p>
			</CardContent>
		</Card>
	{:else}
		<Card>
			<CardContent class="p-0">
				<div class="divide-y">
					{#each filteredLogs as log (log.id)}
						<div class="flex items-start gap-4 p-4 hover:bg-muted/50">
							<div class={getActionColor(log.action)}>
								<FileText class="h-5 w-5" />
							</div>

							<div class="flex-1 space-y-1">
								<div class="flex items-center gap-2">
									<Badge class={getActionColor(log.action)} variant="outline">
										{log.action}
									</Badge>
									<span class="text-sm text-muted-foreground">{log.resource}</span>
								</div>
								<div class="flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
									<span>Usuario: {log.user_id}</span>
									<span>IP: {log.ip_address}</span>
									<span>{formatDate(log.created_at)}</span>
								</div>
								{#if log.details && Object.keys(log.details).length > 0}
									<pre class="mt-2 max-w-md overflow-auto rounded bg-muted p-2 text-xs">
										{JSON.stringify(log.details, null, 2)}
									</pre>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			</CardContent>
		</Card>
	{/if}
</div>
