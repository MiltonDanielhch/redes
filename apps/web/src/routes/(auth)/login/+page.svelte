<!-- Ubicación: `apps/web/src/routes/(auth)/login/+page.svelte` -->
<!-- Descripción: Página de inicio de sesión con validación de credenciales -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0008 (PASETO) -->
<script lang="ts">
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { LoginSchema } from '$lib/validation/schemas';
	import { login } from '$lib/api/auth';
	import { toast } from 'svelte-sonner';

	let email = $state('');
	let password = $state('');
	let loading = $state(false);
	let errors = $state<Record<string, string>>({});

	async function handleSubmit() {
		errors = {};

		const result = LoginSchema.safeParse({ email, password });
		if (!result.success) {
			const fieldErrors: Record<string, string> = {};
			for (const issue of result.error.issues) {
				const path = issue.path[0] as string;
				fieldErrors[path] = issue.message;
			}
			errors = fieldErrors;
			return;
		}

		loading = true;
		try {
			await login({ email, password });
			toast.success('Sesión iniciada');
			goto('/dashboard');
		} catch (err) {
			toast.error(err instanceof Error ? err.message : 'Error al iniciar sesión');
		} finally {
			loading = false;
		}
	}
</script>

<Card class="shadow-xl">
	<CardHeader class="space-y-1">
		<CardTitle class="text-2xl">Iniciar Sesión</CardTitle>
		<CardDescription>Ingresa tus credenciales para acceder al sistema</CardDescription>
	</CardHeader>
	<CardContent>
		<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
			<div class="grid gap-4">
				<div class="grid gap-2">
					<Label for="email">Correo electrónico</Label>
					<Input
						id="email"
						type="email"
						placeholder="correo@ejemplo.com"
						bind:value={email}
						class={errors.email ? 'border-destructive' : ''}
					/>
					{#if errors.email}
						<p class="text-sm text-destructive">{errors.email}</p>
					{/if}
				</div>
				<div class="grid gap-2">
					<Label for="password">Contraseña</Label>
					<Input
						id="password"
						type="password"
						placeholder="••••••••"
						bind:value={password}
						class={errors.password ? 'border-destructive' : ''}
					/>
					{#if errors.password}
						<p class="text-sm text-destructive">{errors.password}</p>
					{/if}
				</div>
				<Button type="submit" class="w-full" disabled={loading}>
					{loading ? 'Iniciando sesión...' : 'Iniciar Sesión'}
				</Button>
			</div>
		</form>
	</CardContent>
	<CardFooter class="flex flex-col gap-4">
		<div class="text-sm text-muted-foreground">
			¿No tienes cuenta? <a href="/register" class="text-primary hover:underline">Regístrate</a>
		</div>
	</CardFooter>
</Card>
