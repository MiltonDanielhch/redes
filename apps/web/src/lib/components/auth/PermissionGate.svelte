<!-- Ubicación: `apps/web/src/lib/components/auth/PermissionGate.svelte` -->
<!-- Descripción: Wrapper que muestra/oculta contenido según permisos del usuario -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0006 (RBAC) -->
<script lang="ts">
	import { auth } from '$lib/stores/auth.svelte';

	interface Props {
		permission: string;
		show?: boolean;
	}

	let { permission, show = false }: Props = $props();

	let hasAccess = $derived(auth.hasPermission(permission));
</script>

{#if hasAccess || show}
	<slot />
{/if}
