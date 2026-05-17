// Ubicación: `apps/web/src/lib/stores/auth.svelte.ts`
//
// Descripción: Store de autenticación global con Svelte 5 Runes.
//              Maneja estado de usuario, tokens PASETO y permisos.
//              Persistencia en localStorage para tokens.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0008 (PASETO)

import type { User } from '$lib/generated/api-types';

interface AuthState {
	user: User | null;
	accessToken: string | null;
	refreshToken: string | null;
}

function createAuthStore() {
	let user = $state<User | null>(null);
	let accessToken = $state<string | null>(null);
	let refreshToken = $state<string | null>(null);

	const isLoggedIn = $derived(user !== null && accessToken !== null);

	const isAdmin = $derived(user?.roles?.includes('admin') ?? false);

	function hasPermission(permission: string): boolean {
		if (!user?.permissions) return false;
		return user.permissions.includes(permission);
	}

	function setAuth(authState: AuthState) {
		user = authState.user;
		accessToken = authState.accessToken;
		refreshToken = authState.refreshToken;

		if (typeof window !== 'undefined') {
			if (authState.accessToken) {
				localStorage.setItem('access_token', authState.accessToken);
			}
			if (authState.refreshToken) {
				localStorage.setItem('refresh_token', authState.refreshToken);
			}
		}
	}

	function clearAuth() {
		user = null;
		accessToken = null;
		refreshToken = null;

		if (typeof window !== 'undefined') {
			localStorage.removeItem('access_token');
			localStorage.removeItem('refresh_token');
			window.location.href = '/login';
		}
	}

	function hydrateFromStorage() {
		if (typeof window !== 'undefined') {
			accessToken = localStorage.getItem('access_token');
			refreshToken = localStorage.getItem('refresh_token');
		}
	}

	return {
		get user() { return user; },
		get accessToken() { return accessToken; },
		get refreshToken() { return refreshToken; },
		get isLoggedIn() { return isLoggedIn; },
		get isAdmin() { return isAdmin; },
		setAuth,
		clearAuth,
		hydrateFromStorage,
		hasPermission,
	};
}

export const auth = createAuthStore();
