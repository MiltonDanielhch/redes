// Ubicación: `apps/web/src/lib/validation/schemas.ts`
//
// Descripción: Schemas de validación con Zod para formularios y API requests.
//              LoginSchema, RegisterSchema, DeviceFormSchema, etc.
//
// ADRs relacionados: 0017 (Frontend SvelteKit)

import { z } from 'zod';

export const LoginSchema = z.object({
	email: z.string().email(),
	password: z.string().min(8),
});

export const RegisterSchema = z.object({
	email: z.string().email(),
	password: z.string().min(8),
	nombre: z.string().min(2).max(100),
});

export const SedeFormSchema = z.object({
	nombre: z.string().min(2).max(200),
	ubicacion: z.string().min(2).max(200),
	secretaria: z.string().min(2).max(200),
});

export const DeviceFormSchema = z.object({
	hostname: z.string().min(1).max(100),
	ip_address: z.string().ip(),
	device_type: z.string().min(1).max(50),
	sede_id: z.string().uuid(),
});

export const AlertFilterSchema = z.object({
	status: z.enum(['active', 'acknowledged', 'resolved']).optional(),
	severity: z.enum(['Critical', 'High', 'Medium', 'Low']).optional(),
	from_date: z.string().datetime().optional(),
	to_date: z.string().datetime().optional(),
});

export const UserFormSchema = z.object({
	email: z.string().email(),
	password: z.string().min(8).optional(),
	nombre: z.string().min(2).max(100),
	role: z.enum(['admin', 'operator', 'viewer']),
});

export type LoginInput = z.infer<typeof LoginSchema>;
export type RegisterInput = z.infer<typeof RegisterSchema>;
export type SedeFormInput = z.infer<typeof SedeFormSchema>;
export type DeviceFormInput = z.infer<typeof DeviceFormSchema>;
export type AlertFilterInput = z.infer<typeof AlertFilterSchema>;
export type UserFormInput = z.infer<typeof UserFormSchema>;
