<!-- Ubicación: `apps/web/src/routes/(dashboard)/+layout.svelte` -->
<!-- Descripción: Layout principal del dashboard con sidebar y topbar -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0006 (RBAC) -->
<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import Topbar from '$lib/components/layout/Topbar.svelte';
	import { auth } from '$lib/stores/auth.svelte';
	import { onMount } from 'svelte';

	onMount(() => {
		if (!auth.isLoggedIn) {
			goto('/login');
		}
	});
</script>

<div class="flex h-screen overflow-hidden">
	<Sidebar />

	<div class="flex flex-1 flex-col overflow-hidden">
		<Topbar user={auth.user} />

		<main class="flex-1 overflow-y-auto bg-background p-6">
			<slot />
		</main>
	</div>
</div>
