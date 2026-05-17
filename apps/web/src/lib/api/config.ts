/**
 * Ubicación: `apps/web/src/lib/api/config.ts`
 *
 * Descripción: Configuración de la API para el frontend.
 *
 * ADRs relacionados: 0016 (OpenAPI), 0017 (SvelteKit)
 */

export const API_BASE_URL =
	import.meta.env.VITE_API_URL || 'http://localhost:8080';