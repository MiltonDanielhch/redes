<!-- Ubicación: `apps/web/src/routes/(auth)/register/+page.svelte` -->
<!-- Descripción: Página de registro de nuevos usuarios -->
<!-- ADRs relacionados: 0017 (Frontend SvelteKit), 0008 (PASETO) -->
<script lang="ts">
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { RegisterSchema } from '$lib/validation/schemas';
	import { register } from '$lib/api/auth';
	import { toast } from 'svelte-sonner';

	let nombre = $state('');
	let email = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let loading = $state(false);
	let errors = $state<Record<string, string>>({});

	async function handleSubmit() {
		errors = {};

		if (password !== confirmPassword) {
			errors.confirmPassword = 'Las contraseñas no coinciden';
			return;
		}

		const result = RegisterSchema.safeParse({ nombre, email, password });
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
			await register({ nombre, email, password });
			toast.success('Cuenta creada exitosamente');
			goto('/dashboard');
		} catch (err) {
			toast.error(err instanceof Error ? err.message : 'Error al registrarse');
		} finally {
			loading = false;
		}
	}
</script>

<Card class="shadow-xl">
	<CardHeader class="space-y-1">
		<CardTitle class="text-2xl">Crear Cuenta</CardTitle>
		<CardDescription>Regístrate para acceder al sistema de monitoreo</CardDescription>
	</CardHeader>
	<CardContent>
		<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
			<div class="grid gap-4">
				<div class="grid gap-2">
					<Label for="nombre">Nombre completo</Label>
					<Input
						id="nombre"
						type="text"
						placeholder="Juan Pérez"
						bind:value={nombre}
						class={errors.nombre ? 'border-destructive' : ''}
					/>
					{#if errors.nombre}
						<p class="text-sm text-destructive">{errors.nombre}</p>
					{/if}
				</div>
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
				<div class="grid gap-2">
					<Label for="confirmPassword">Confirmar contraseña</Label>
					<Input
						id="confirmPassword"
						type="password"
						placeholder="••••••••"
						bind:value={confirmPassword}
						class={errors.confirmPassword ? 'border-destructive' : ''}
					/>
					{#if errors.confirmPassword}
						<p class="text-sm text-destructive">{errors.confirmPassword}</p>
					{/if}
				</div>
				<Button type="submit" class="w-full" disabled={loading}>
					{loading ? 'Creando cuenta...' : 'Crear Cuenta'}
				</Button>
			</div>
		</form>
	</CardContent>
	<CardFooter class="flex flex-col gap-4">
		<div class="text-sm text-muted-foreground">
			¿Ya tienes cuenta? <a href="/login" class="text-primary hover:underline">Inicia Sesión</a>
		</div>
	</CardFooter>
</Card>
