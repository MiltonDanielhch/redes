<!-- Ubicación: `apps/web/src/routes/+page.svelte` -->
<!-- Descripción: Página raíz que redirige a /login o /dashboard según estado de auth -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit) -->
<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { auth } from '$lib/stores/auth.svelte';

	onMount(() => {
		if (auth.isLoggedIn) {
			goto('/dashboard');
		} else {
			goto('/login');
		}
	});
</script>

<div class="flex h-screen items-center justify-center">
	<p>Redirigiendo...</p>
</div>
