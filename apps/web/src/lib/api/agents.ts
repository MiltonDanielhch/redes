// Ubicación: `apps/web/src/lib/api/agents.ts`
//
// Descripción: API module para gestión de agentes de monitoreo SNMP/Netflow.
//              listAgents, getAgent, updateAgent, getAgentStatus.
//
// ADRs relacionados: 0017 (Frontend SvelteKit), 0022 (Agentes)

import { apiClient } from './client';
import type { AgentResponse } from '$lib/generated/api-types';

export interface AgentStatus {
	agent_id: string;
	status: 'online' | 'offline' | 'syncing';
	last_seen_at: string;
	metrics_collected: number;
	errors: string[];
}

export async function listAgents(): Promise<AgentResponse[]> {
	return apiClient.get<AgentResponse[]>('/api/v1/agents');
}

export async function getAgent(id: string): Promise<AgentResponse> {
	return apiClient.get<AgentResponse>(`/api/v1/agents/${id}`);
}

export async function getAgentStatus(id: string): Promise<AgentStatus> {
	return apiClient.get<AgentStatus>(`/api/v1/agents/${id}/status`);
}

export async function updateAgent(
	id: string,
	data: Partial<AgentResponse>
): Promise<AgentResponse> {
	return apiClient.put<AgentResponse>(`/api/v1/agents/${id}`, data);
}

export async function restartAgent(id: string): Promise<void> {
	return apiClient.post<void>(`/api/v1/agents/${id}/restart`);
}
