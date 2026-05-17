#!/usr/bin/env node

import { readFileSync, writeFileSync } from 'fs';
import { join } from 'path';

const OPENAPI_URL = process.env.OPENAPI_URL || 'http://localhost:8080/api-docs.json';
const OUTPUT_PATH = join(import.meta.dirname, 'api-types.ts');

async function generateTypes() {
	console.log('Fetching OpenAPI spec from:', OPENAPI_URL);

	const response = await fetch(OPENAPI_URL);
	if (!response.ok) {
		throw new Error(`Failed to fetch OpenAPI spec: ${response.status} ${response.statusText}`);
	}

	const openapi = await response.json();
	console.log('OpenAPI spec fetched successfully');

	const schemas = openapi.components?.schemas || {};
	const types: string[] = [];

	for (const [name, schema] of Object.entries(schemas)) {
		const typeName = toPascalCase(name);
		const typeDef = convertSchemaToType(schema, typeName);
		if (typeDef) {
			types.push(typeDef);
		}
	}

	const output = `// Auto-generated from OpenAPI spec
// Run: node scripts/generate-types.ts

${types.join('\n\n')}

export type SchemaNames = ${Object.keys(schemas).map((s) => `'${s}'`).join(' | ')};
`;

	writeFileSync(OUTPUT_PATH, output, 'utf-8');
	console.log('Types generated at:', OUTPUT_PATH);
}

function toPascalCase(str: string): string {
	return str
		.replace(/[-_](\w)/g, (_, c) => c.toUpperCase())
		.replace(/^(.)/, (_, c) => c.toUpperCase());
}

function convertSchemaToType(schema: Record<string, unknown>, name: string): string | null {
	if (!schema) return null;

	const props = schema.properties as Record<string, Record<string, unknown>> | undefined;
	if (!props) {
		if (schema.type === 'string') return `export type ${name} = string;`;
		if (schema.type === 'number') return `export type ${name} = number;`;
		if (schema.type === 'boolean') return `export type ${name} = boolean;`;
		return null;
	}

	const lines: string[] = [];
	for (const [propName, prop] of Object.entries(props)) {
		const optional = schema.required && !(schema.required as string[]).includes(propName);
		const tsType = schemaTypeToTs(prop);
		const comment = prop.description ? `// ${prop.description}\n    ` : '';
		lines.push(`${comment}${propName}${optional ? '?' : ''}: ${tsType};`);
	}

	return `export interface ${name} {\n    ${lines.join('\n    ')}\n}`;
}

function schemaTypeToTs(prop: Record<string, unknown>): string {
	if (prop.$ref) {
		return toPascalCase(prop.$ref.split('/').pop() || '');
	}

	const type = prop.type as string;
	if (type === 'string') {
		if (prop.format === 'date-time') return 'string';
		if (prop.format === 'uuid') return 'string';
		if (prop.enum) return prop.enum.map((v) => `'${v}'`).join(' | ');
		return 'string';
	}

	if (type === 'number' || type === 'integer') return 'number';
	if (type === 'boolean') return 'boolean';
	if (type === 'array') {
		const itemType = prop.items ? schemaTypeToTs(prop.items as Record<string, unknown>) : 'unknown';
		return `${itemType}[]`;
	}

	if (type === 'object' || !type) {
		if (prop.properties) {
			const nested = Object.entries(prop.properties)
				.map(([k, v]) => `${k}: ${schemaTypeToTs(v as Record<string, unknown>)}`)
				.join('; ');
			return `{ ${nested} }`;
		}
		return 'Record<string, unknown>';
	}

	return 'unknown';
}

generateTypes().catch((err) => {
	console.error('Error generating types:', err);
	process.exit(1);
});
