// @ts-nocheck
// Ubicación: `apps/web/src/routes/(auth)/+layout.ts`
//
// Descripción: Load function que redirige si el usuario ya está logueado.
//              Solo permite acceso si NO está autenticado.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0008 (PASETO)

import type { LayoutLoad } from './$types';
import { checkAuth } from '$lib/routeGuard';

export const load = () => {
	checkAuth({ requireLogout: true });
};
;null as any as LayoutLoad;