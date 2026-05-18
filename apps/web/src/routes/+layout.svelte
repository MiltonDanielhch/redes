<!-- Ubicación: `apps/web/src/routes/+layout.svelte` -->
<!-- Descripción: Root layout con QueryClientProvider y Toaster para toda la app -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit) -->
<script lang="ts">
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
	import { Toaster } from 'svelte-sonner';
	import OfflineBanner from '$lib/components/network/OfflineBanner.svelte';
	import '../app.css';

	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: 5 * 60 * 1000,
				retry: 1,
				refetchOnWindowFocus: false,
			},
		},
	});
</script>

<QueryClientProvider client={queryClient}>
	<OfflineBanner />
	<Toaster position="top-right" />
	<slot />
</QueryClientProvider>
