/**
 * Ubicación: `apps/web/src/lib/types/index.ts`
 *
 * Descripción: Tipos TypeScript para el frontend.
 *              Sincronizados con las entidades de dominio Rust.
 *
 * ADRs relacionados: 0016 (OpenAPI), 0017 (SvelteKit)
 */

export type User = {
	id: string;
	email: string;
	name: string;
	role: string;
};

export type Sede = {
	id: string;
	nombre: string;
	ubicacion: string;
	secretaria: string;
};

export type Device = {
	id: string;
	sede_id: string;
	hostname: string;
	ip_address: string;
	mac_address: string | null;
	device_type: DeviceType;
	status: DeviceStatus;
};

export type DeviceType =
	| 'Switch'
	| 'AccessPoint'
	| 'Router'
	| 'Firewall'
	| 'Server'
	| 'Ups'
	| 'Camera';

export type DeviceStatus = 'Active' | 'Offline' | 'Maintenance';

export type MetricReading = {
	id: string;
	device_id: string;
	bandwidth_rx_kbps: number;
	bandwidth_tx_kbps: number;
	latency_ms: number;
	packet_loss: number;
	anomaly: boolean;
	recorded_at: string;
};

export type Alert = {
	id: string;
	device_id: string;
	tipo: AlertType;
	severidad: AlertSeverity;
	mensaje: string;
	created_at: string;
};

export type AlertType = ' connectivity' | 'bandwidth' | 'intrusion' | 'system';
export type AlertSeverity = 'critical' | 'high' | 'medium' | 'low';