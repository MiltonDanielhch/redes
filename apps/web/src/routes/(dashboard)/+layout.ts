// Ubicación: `apps/web/src/routes/(dashboard)/+layout.ts`
//
// Descripción: Load function que protege todas las rutas del dashboard.
//              Verifica que el usuario esté autenticado.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0008 (PASETO)

import type { LayoutLoad } from './$types';
import { checkAuth } from '$lib/routeGuard';

export const load: LayoutLoad = () => {
	checkAuth({ requireLogin: true });

	return {
		user: {
			id: crypto.randomUUID(),
			email: 'demo@example.com',
			nombre: 'Usuario Demo',
			roles: ['admin'],
		},
	};
};
