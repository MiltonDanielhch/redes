// Ubicación: `apps/web/src/lib/generated/api-types.ts`
//
// Descripción: Tipos TypeScript auto-generados desde los DTOs del backend.
//              NO EDITAR MANUALMENTE - regenerar con `pnpm generate-types`.
//
// ADRs relacionados: 0016 (OpenAPI), 0017 (Frontend SvelteKit)

export interface CreateSedeRequest {
	nombre: string;
	ubicacion: string;
	secretaria: string;
}

export interface UpdateSedeRequest {
	nombre?: string;
	ubicacion?: string;
	secretaria?: string;
}

export interface SedeResponse {
	id: string;
	nombre: string;
	ubicacion: string;
	secretaria: string;
	created_at: string;
	updated_at: string;
}

export interface CreateDeviceRequest {
	hostname: string;
	ip_address: string;
	device_type: string;
	sede_id: string;
}

export interface UpdateDeviceRequest {
	hostname?: string;
	ip_address?: string;
	device_type?: string;
	status?: string;
}

export interface DeviceResponse {
	id: string;
	hostname: string;
	ip_address: string;
	mac_address?: string;
	device_type: string;
	status: string;
	sede_id: string;
	last_seen_at?: string;
	created_at: string;
}

export interface CreateAlertRequest {
	alertType: string;
	severity: string;
	deviceId?: string;
	message: string;
	details?: string;
}

export interface AcknowledgeAlertRequest {
	userId: string;
}

export interface AlertResponse {
	id: string;
	alertType: string;
	severity: string;
	deviceId?: string;
	message: string;
	details?: string;
	status: string;
	acknowledgedBy?: string;
	acknowledgedAt?: string;
	createdAt: string;
	updatedAt: string;
}

export interface AlertListResponse {
	alerts: AlertResponse[];
	total: number;
}

export interface ApiErrorResponse {
	code: string;
	message: string;
	details?: Record<string, unknown>;
}

export interface User {
	id: string;
	email: string;
	nombre: string;
	roles: string[];
	permissions: string[];
	created_at: string;
}

export interface LoginRequest {
	email: string;
	password: string;
}

export interface LoginResponse {
	user: User;
	access_token: string;
	refresh_token: string;
}

export interface RegisterRequest {
	email: string;
	password: string;
	nombre: string;
}

export interface HealthResponse {
	overall_healthy: boolean;
	checks: Record<string, boolean>;
}

export interface PaginationParams {
	page?: number;
	limit?: number;
	offset?: number;
}

export interface ApiResponse<T> {
	data: T;
	message?: string;
}

export interface MetricReading {
	id: string;
	device_id: string;
	metric_type: string;
	value: number;
	unit: string;
	timestamp: string;
}

export interface IntrusionEvent {
	id: string;
	source_ip: string;
	mac_address: string;
	intrusion_type: string;
	severity: string;
	status: string;
	detected_at: string;
	resolved_at?: string;
	device_id?: string;
}

export interface CreateIntrusionRequest {
	source_ip: string;
	mac_address: string;
	intrusion_type: string;
	severity: string;
	device_id?: string;
}

export interface CreateUserRequest {
	email: string;
	password: string;
	nombre: string;
	role: string;
}

export interface UpdateUserRequest {
	email?: string;
	nombre?: string;
	role?: string;
	password?: string;
}

export interface AuditLogEntry {
	id: string;
	user_id: string;
	action: string;
	resource: string;
	details: Record<string, unknown>;
	ip_address: string;
	created_at: string;
}
