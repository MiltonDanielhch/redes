<!-- Ubicación: `apps/web/src/routes/(auth)/+layout.svelte` -->
<!-- Descripción: Layout para páginas de autenticación (login, register) sin sidebar -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit) -->
<script lang="ts">
	import { ShieldAlert } from 'lucide-svelte';
</script>

<div class="flex min-h-screen items-center justify-center bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800">
	<div class="w-full max-w-md p-8">
		<div class="mb-8 flex flex-col items-center text-center">
			<div
				class="mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-primary text-primary-foreground shadow-lg"
			>
				<ShieldAlert class="h-8 w-8" />
			</div>
			<h1 class="text-2xl font-bold tracking-tight">Monitoreo Regional</h1>
			<p class="text-muted-foreground">Gobernación del Beni</p>
		</div>

		<slot />
	</div>
</div>
