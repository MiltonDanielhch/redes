// Ubicación: `apps/web/src/lib/api/auth.ts`
//
// Descripción: Módulo de autenticación con funciones de login, logout, register
//              y refresh de tokens PASETO. Integración con auth store.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0008 (PASETO)

import { apiClient } from './client';
import type { LoginRequest, LoginResponse, RegisterRequest, User } from '$lib/generated/api-types';
import { auth } from '$lib/stores/auth.svelte';

export async function login(credentials: LoginRequest): Promise<void> {
	const response = await apiClient.post<LoginResponse>('/api/v1/auth/login', credentials);

	auth.setAuth({
		user: response.user,
		accessToken: response.access_token,
		refreshToken: response.refresh_token,
	});
}

export async function register(data: RegisterRequest): Promise<void> {
	const response = await apiClient.post<LoginResponse>('/api/v1/auth/register', data);

	auth.setAuth({
		user: response.user,
		accessToken: response.access_token,
		refreshToken: response.refresh_token,
	});
}

export async function logout(): Promise<void> {
	try {
		await apiClient.post('/api/v1/auth/logout');
	} finally {
		auth.clearAuth();
	}
}

export async function refreshTokens(): Promise<boolean> {
	const refreshToken = auth.refreshToken;
	if (!refreshToken) return false;

	try {
		const response = await apiClient.post<LoginResponse>('/api/v1/auth/refresh', {
			refresh_token: refreshToken,
		});

		auth.setAuth({
			user: response.user,
			accessToken: response.access_token,
			refreshToken: response.refresh_token,
		});

		return true;
	} catch {
		auth.clearAuth();
		return false;
	}
}

export async function getCurrentUser(): Promise<User> {
	return apiClient.get<User>('/api/v1/auth/me');
}
