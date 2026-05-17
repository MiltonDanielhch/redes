// Ubicación: `apps/web/src/lib/types/ui.ts`
//
// Descripción: Tipos TypeScript para estado UI local y componentes.
//              No son tipos del dominio backend, solo para frontend.
//
// ADRs relacionados: 0017 (Frontend SvelteKit)

export interface Toast {
	id: string;
	type: 'success' | 'error' | 'info' | 'warning';
	title: string;
	description?: string;
	duration?: number;
}

export interface ModalState {
	isOpen: boolean;
	title?: string;
	description?: string;
	onConfirm?: () => void;
	onCancel?: () => void;
}

export interface FilterState {
	search?: string;
	status?: string;
	severity?: string;
	dateRange?: {
		from?: Date;
		to?: Date;
	};
}

export interface ChartConfig {
	colors?: string[];
	showLegend?: boolean;
	showGrid?: boolean;
	animated?: boolean;
}

export interface PaginationState {
	page: number;
	limit: number;
	total: number;
}

export interface SortState {
	column: string;
	direction: 'asc' | 'desc';
}

export interface TableState {
	filters: FilterState;
	sort: SortState;
	pagination: PaginationState;
}
