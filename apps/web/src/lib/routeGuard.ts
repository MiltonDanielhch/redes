// Ubicación: `apps/web/src/lib/routeGuard.ts`
//
// Descripción: Utilidades para protección de rutas y redirección auth.
//              checkAuth() para verificar sesión en load functions.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0008 (PASETO)

import { redirect } from '@sveltejs/kit';
import { auth } from '$lib/stores/auth.svelte';

export interface AuthGuardOptions {
	requireLogin?: boolean;
	requireLogout?: boolean;
	requiredPermissions?: string[];
	requiredRoles?: string[];
}

export function checkAuth(options: AuthGuardOptions = {}) {
	if (options.requireLogin) {
		if (!auth.isLoggedIn) {
			throw redirect(302, '/login');
		}

		if (options.requiredPermissions?.length) {
			const hasAllPermissions = options.requiredPermissions.every((p) =>
				auth.hasPermission(p)
			);
			if (!hasAllPermissions) {
				throw redirect(302, '/dashboard?error=forbidden');
			}
		}

		if (options.requiredRoles?.length) {
			const hasRole = options.requiredRoles.some((r) =>
				auth.user?.roles.includes(r)
			);
			if (!hasRole) {
				throw redirect(302, '/dashboard?error=forbidden');
			}
		}
	}

	if (options.requireLogout && auth.isLoggedIn) {
		throw redirect(302, '/dashboard');
	}

	return auth;
}
