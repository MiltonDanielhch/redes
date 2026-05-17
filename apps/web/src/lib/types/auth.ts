// Ubicación: `apps/web/src/lib/types/auth.ts`
//
// Descripción: Tipos TypeScript para estado de autenticación local.
//              No son tipos del backend, solo para el frontend.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0008 (PASETO)

export interface AuthState {
	isAuthenticated: boolean;
	user: {
		id: string;
		email: string;
		nombre: string;
		roles: string[];
	} | null;
}

export interface Permission {
	resource: string;
	actions: ('create' | 'read' | 'update' | 'delete')[];
}
