// Ubicación: `apps/web/src/lib/api/users.ts`
//
// Descripción: API module para administración de usuarios y auditoría.
//              CRUD de usuarios, listAuditLogs.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0006 (RBAC)

import { apiClient } from './client';
import type {
	User,
	CreateUserRequest,
	UpdateUserRequest,
	AuditLogEntry,
} from '$lib/generated/api-types';

export async function listUsers(): Promise<User[]> {
	return apiClient.get<User[]>('/api/v1/users');
}

export async function getUser(id: string): Promise<User> {
	return apiClient.get<User>(`/api/v1/users/${id}`);
}

export async function createUser(data: CreateUserRequest): Promise<User> {
	return apiClient.post<User>('/api/v1/users', data);
}

export async function updateUser(id: string, data: UpdateUserRequest): Promise<User> {
	return apiClient.put<User>(`/api/v1/users/${id}`, data);
}

export async function deleteUser(id: string): Promise<void> {
	return apiClient.delete<void>(`/api/v1/users/${id}`);
}

export async function listAuditLogs(params?: {
	user_id?: string;
	action?: string;
	limit?: number;
}): Promise<AuditLogEntry[]> {
	const searchParams = new URLSearchParams();
	if (params?.user_id) searchParams.set('user_id', params.user_id);
	if (params?.action) searchParams.set('action', params.action);
	if (params?.limit) searchParams.set('limit', params.limit.toString());

	const query = searchParams.toString();
	return apiClient.get<AuditLogEntry[]>(`/api/v1/audit${query ? `?${query}` : ''}`);
}
