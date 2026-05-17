<!-- Ubicación: `apps/web/src/lib/components/dashboard/KpiCard.svelte` -->
<!-- Descripción: Card de métrica KPI con título, valor, icono, tendencia y enlace -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit) -->
<script lang="ts">
	import { TrendingUp, TrendingDown, ExternalLink } from 'lucide-svelte';
	import { Card, CardContent } from '$lib/components/ui/card';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { cn } from '$lib/utils';

	interface Props {
		title: string;
		value: string | number;
		icon?: typeof TrendingUp;
		trend?: {
			value: number;
			direction: 'up' | 'down';
		};
		description?: string;
		href?: string;
		loading?: boolean;
	}

	let {
		title,
		value,
		icon: Icon,
		trend,
		description,
		href,
		loading = false
	}: Props = $props();

	const trendColors = {
		up: 'text-green-500',
		down: 'text-red-500'
	};
</script>

<Card class="relative overflow-hidden">
	<CardContent class="p-6">
		{#if loading}
			<div class="space-y-3">
				<Skeleton class="h-4 w-24" />
				<Skeleton class="h-8 w-20" />
				<Skeleton class="h-3 w-32" />
			</div>
		{:else}
			<div class="flex items-start justify-between">
				<div class="space-y-1">
					<p class="text-sm font-medium text-muted-foreground">{title}</p>
					<p class="text-3xl font-bold tracking-tight">{value}</p>
					{#if description}
						<p class="text-xs text-muted-foreground">{description}</p>
					{/if}
				</div>
				{#if Icon}
					<div
						class="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10 text-primary"
					>
						<Icon class="h-6 w-6" />
					</div>
				{/if}
			</div>

			{#if trend}
				<div class="mt-4 flex items-center gap-1 text-sm">
					{#if trend.direction === 'up'}
						<TrendingUp class={cn('h-4 w-4', trendColors.up)} />
						<span class={cn(trendColors.up, 'font-medium')}>+{trend.value}%</span>
					{:else}
						<TrendingDown class={cn('h-4 w-4', trendColors.down)} />
						<span class={cn(trendColors.down, 'font-medium')}>{trend.value}%</span>
					{/if}
					<span class="text-muted-foreground">vs anterior</span>
				</div>
			{/if}

			{#if href}
				<a
					{href}
					class="absolute inset-0"
					aria-label="Ver detalles de {title}"
				>
					<span class="sr-only">Ver detalles</span>
				</a>
			{/if}
		{/if}
	</CardContent>
</Card>
