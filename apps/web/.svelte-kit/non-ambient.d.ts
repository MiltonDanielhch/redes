
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
		RouteId(): "/(dashboard)" | "/(auth)" | "/" | "/(dashboard)/dashboard" | "/(dashboard)/devices" | "/(dashboard)/devices/[id]" | "/(auth)/login" | "/(auth)/register" | "/(dashboard)/sedes";
		RouteParams(): {
			"/(dashboard)/devices/[id]": { id: string }
		};
		LayoutParams(): {
			"/(dashboard)": { id?: string };
			"/(auth)": Record<string, never>;
			"/": { id?: string };
			"/(dashboard)/dashboard": Record<string, never>;
			"/(dashboard)/devices": { id?: string };
			"/(dashboard)/devices/[id]": { id: string };
			"/(auth)/login": Record<string, never>;
			"/(auth)/register": Record<string, never>;
			"/(dashboard)/sedes": Record<string, never>
		};
		Pathname(): "/" | "/dashboard" | "/devices" | `/devices/${string}` & {} | "/login" | "/register" | "/sedes";
		ResolvedPathname(): `${"" | `/${string}`}${ReturnType<AppTypes['Pathname']>}`;
		Asset(): "/favicon.svg" | string & {};
	}
}