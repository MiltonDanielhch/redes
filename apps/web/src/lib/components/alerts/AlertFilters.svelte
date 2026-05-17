<!-- Ubicación: `apps/web/src/lib/components/alerts/AlertFilters.svelte` -->
<!-- Descripción: Panel de filtros para alertas con rango de fechas y severidad -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0020 (Monitoreo Regional) -->
<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { cn } from '$lib/utils';

	interface Props {
		selectedSeverity?: string[];
		selectedStatuses?: string[];
		onSeverityChange?: (severities: string[]) => void;
		onStatusChange?: (statuses: string[]) => void;
		onReset?: () => void;
	}

	let {
		selectedSeverity = [],
		selectedStatuses = [],
		onSeverityChange,
		onStatusChange,
		onReset
	}: Props = $props();

	const severities = ['Critical', 'High', 'Medium', 'Low'];
	const statuses = [
		{ value: 'active', label: 'Activa', color: 'bg-red-500' },
		{ value: 'acknowledged', label: 'Reconocida', color: 'bg-yellow-500' },
		{ value: 'resolved', label: 'Resuelta', color: 'bg-green-500' },
	];

	function toggleSeverity(severity: string) {
		const newList = selectedSeverity.includes(severity)
			? selectedSeverity.filter((s) => s !== severity)
			: [...selectedSeverity, severity];
		onSeverityChange?.(newList);
	}

	function toggleStatus(status: string) {
		const newList = selectedStatuses.includes(status)
			? selectedStatuses.filter((s) => s !== status)
			: [...selectedStatuses, status];
		onStatusChange?.(newList);
	}
</script>

<Card>
	<CardHeader class="pb-3">
		<div class="flex items-center justify-between">
			<CardTitle class="text-base">Filtros</CardTitle>
			{#if selectedSeverity.length > 0 || selectedStatuses.length > 0}
				<Button variant="ghost" size="sm" onclick={onReset}>
					Limpiar
				</Button>
			{/if}
		</div>
	</CardHeader>
	<CardContent class="space-y-6">
		<div class="space-y-3">
			<label class="text-sm font-medium">Severidad</label>
			<div class="flex flex-wrap gap-2">
				{#each severities as severity}
					<button
						type="button"
						class="rounded-full px-3 py-1 text-sm transition-colors {selectedSeverity.includes(severity)
							? 'bg-primary text-primary-foreground'
							: 'bg-muted hover:bg-muted/80'}"
						onclick={() => toggleSeverity(severity)}
					>
						{severity}
					</button>
				{/each}
			</div>
		</div>

		<div class="space-y-3">
			<label class="text-sm font-medium">Estado</label>
			<div class="space-y-2">
				{#each statuses as status}
					<button
						type="button"
						class="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left transition-colors hover:bg-muted {selectedStatuses.includes(status.value) ? 'bg-accent' : ''}"
						onclick={() => toggleStatus(status.value)}
					>
						<div class={cn('h-2 w-2 rounded-full', status.color)} />
						<span class="flex-1 text-sm">{status.label}</span>
						{#if selectedStatuses.includes(status.value)}
							<Badge variant="secondary" class="ml-auto">✓</Badge>
						{/if}
					</button>
				{/each}
			</div>
		</div>
	</CardContent>
</Card>
