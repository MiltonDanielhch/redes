
// this file is generated — do not edit it


declare module "svelte/elements" {
	export interface HTMLAttributes<T> {
		'data-sveltekit-keepfocus'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-noscroll'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-preload-code'?:
			| true
			| ''
			| 'eager'
			| 'viewport'
			| 'hover'
			| 'tap'
			| 'off'
			| undefined
			| null;
		'data-sveltekit-preload-data'?: true | '' | 'hover' | 'tap' | 'off' | undefined | null;
		'data-sveltekit-reload'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-replacestate'?: true | '' | 'off' | undefined | null;
	}
}

export {};


declare module "$app/types" {
	type MatcherParam<M> = M extends (param : string) => param is (infer U extends string) ? U : string;

	export interface AppTypes {
		RouteId(): "/(dashboard)" | "/(auth)" | "/" | "/(dashboard)/admin" | "/(dashboard)/admin/audit" | "/(dashboard)/admin/users" | "/(dashboard)/alerts" | "/(dashboard)/dashboard" | "/(dashboard)/devices" | "/(dashboard)/devices/[id]" | "/(dashboard)/intrusions" | "/(auth)/login" | "/(dashboard)/metrics" | "/(auth)/register" | "/(dashboard)/sedes" | "/(dashboard)/topology";
		RouteParams(): {
			"/(dashboard)/devices/[id]": { id: string }
		};
		LayoutParams(): {
			"/(dashboard)": { id?: string };
			"/(auth)": Record<string, never>;
			"/": { id?: string };
			"/(dashboard)/admin": Record<string, never>;
			"/(dashboard)/admin/audit": Record<string, never>;
			"/(dashboard)/admin/users": Record<string, never>;
			"/(dashboard)/alerts": Record<string, never>;
			"/(dashboard)/dashboard": Record<string, never>;
			"/(dashboard)/devices": { id?: string };
			"/(dashboard)/devices/[id]": { id: string };
			"/(dashboard)/intrusions": Record<string, never>;
			"/(auth)/login": Record<string, never>;
			"/(dashboard)/metrics": Record<string, never>;
			"/(auth)/register": Record<string, never>;
			"/(dashboard)/sedes": Record<string, never>;
			"/(dashboard)/topology": Record<string, never>
		};
		Pathname(): "/" | "/admin/audit" | "/admin/users" | "/alerts" | "/dashboard" | "/devices" | `/devices/${string}` & {} | "/intrusions" | "/login" | "/metrics" | "/register" | "/sedes" | "/topology";
		ResolvedPathname(): `${"" | `/${string}`}${ReturnType<AppTypes['Pathname']>}`;
		Asset(): "/favicon.svg" | string & {};
	}
}