<!-- Ubicación: `apps/web/src/lib/components/auth/AuthForm.svelte` -->
<!-- Descripción: Componente de formulario de autenticación con validación y estado de carga -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0008 (PASETO) -->
<script lang="ts">
	import { Eye, EyeOff } from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { toast } from 'svelte-sonner';
	import { auth } from '$lib/stores/auth.svelte';

	interface Props {
		email: string;
		password: string;
		loading: boolean;
		errors: Record<string, string>;
		onEmailChange: (value: string) => void;
		onPasswordChange: (value: string) => void;
		onSubmit: () => void;
	}

	let {
		email,
		password,
		loading,
		errors,
		onEmailChange,
		onPasswordChange,
		onSubmit
	}: Props = $props();

	let showPassword = $state(false);
</script>

<form onsubmit={(e) => { e.preventDefault(); onSubmit(); }}>
	<div class="grid gap-4">
		<div class="grid gap-2">
			<Label for="email">Correo electrónico</Label>
			<Input
				id="email"
				type="email"
				placeholder="correo@ejemplo.com"
				value={email}
				oninput={(e) => onEmailChange(e.currentTarget.value)}
				class={errors.email ? 'border-destructive' : ''}
				disabled={loading}
			/>
			{#if errors.email}
				<p class="text-sm text-destructive">{errors.email}</p>
			{/if}
		</div>

		<div class="grid gap-2">
			<Label for="password">Contraseña</Label>
			<div class="relative">
				<Input
					id="password"
					type={showPassword ? 'text' : 'password'}
					placeholder="••••••••"
					value={password}
					oninput={(e) => onPasswordChange(e.currentTarget.value)}
					class={errors.password ? 'border-destructive pr-10' : 'pr-10'}
					disabled={loading}
				/>
				<Button
					type="button"
					variant="ghost"
					size="icon"
					class="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
					onclick={() => (showPassword = !showPassword)}
				>
					{#if showPassword}
						<EyeOff class="h-4 w-4 text-muted-foreground" />
					{:else}
						<Eye class="h-4 w-4 text-muted-foreground" />
					{/if}
				</Button>
			</div>
			{#if errors.password}
				<p class="text-sm text-destructive">{errors.password}</p>
			{/if}
		</div>

		<Button type="submit" class="w-full" disabled={loading}>
			{loading ? 'Iniciando sesión...' : 'Iniciar Sesión'}
		</Button>
	</div>
</form>
