// Ubicación: `apps/web/src/lib/api/client.ts`
//
// Descripción: Cliente HTTP con fetch para comunicación con la API backend.
//              Maneja autenticación con PASETO, retry de tokens, y errores.
//              Implementa el patrón Repository en el frontend.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0008 (PASETO), 0016 (OpenAPI)

import type { ApiErrorResponse } from '$lib/generated/api-types';

export interface ApiClientConfig {
	baseUrl: string;
	getAccessToken: () => string | null;
	onUnauthorized: () => void;
}

export class ApiClient {
	private baseUrl: string;
	private getAccessToken: () => string | null;
	private onUnauthorized: () => void;

	constructor(config: ApiClientConfig) {
		this.baseUrl = config.baseUrl;
		this.getAccessToken = config.getAccessToken;
		this.onUnauthorized = config.onUnauthorized;
	}

	private generateRequestId(): string {
		return `req_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
	}

	private async request<T>(
		method: string,
		path: string,
		body?: unknown,
		options?: RequestInit
	): Promise<T> {
		const accessToken = this.getAccessToken();
		const requestId = this.generateRequestId();

		const headers: Record<string, string> = {
			'Content-Type': 'application/json',
			'X-Request-Id': requestId,
			...options?.headers as Record<string, string>,
		};

		if (accessToken) {
			headers['Authorization'] = `Bearer ${accessToken}`;
		}

		const response = await fetch(`${this.baseUrl}${path}`, {
			method,
			headers,
			body: body ? JSON.stringify(body) : undefined,
			...options,
		});

		if (response.status === 401) {
			this.onUnauthorized();
			throw new Error('Unauthorized');
		}

		if (response.status === 403) {
			window.location.href = '/dashboard';
			throw new Error('Forbidden');
		}

		if (response.status === 500) {
			console.error(`[API Error] 500 on ${method} ${path}:`, await response.text());
			throw new Error('Internal server error');
		}

		if (!response.ok) {
			const errorData: ApiErrorResponse = await response.json().catch(() => ({
				code: 'UNKNOWN_ERROR',
				message: `Request failed with status ${response.status}`,
			}));
			throw new Error(errorData.message || `Request failed with status ${response.status}`);
		}

		return response.json();
	}

	async get<T>(path: string, options?: RequestInit): Promise<T> {
		return this.request<T>('GET', path, undefined, options);
	}

	async post<T>(path: string, body?: unknown, options?: RequestInit): Promise<T> {
		return this.request<T>('POST', path, body, options);
	}

	async put<T>(path: string, body?: unknown, options?: RequestInit): Promise<T> {
		return this.request<T>('PUT', path, body, options);
	}

	async delete<T>(path: string, options?: RequestInit): Promise<T> {
		return this.request<T>('DELETE', path, undefined, options);
	}
}

export const apiClient = new ApiClient({
	baseUrl: import.meta.env.PUBLIC_API_URL || 'http://localhost:8080',
	getAccessToken: () => {
		if (typeof window === 'undefined') return null;
		return localStorage.getItem('access_token');
	},
	onUnauthorized: () => {
		localStorage.removeItem('access_token');
		localStorage.removeItem('refresh_token');
		window.location.href = '/login';
	},
});
