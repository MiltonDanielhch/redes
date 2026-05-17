import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import prettier from 'eslint-config-prettier';
import globals from 'globals';

export default [
	js.configs.recommended,
	...svelte.configs['flat/recommended'],
	prettier,
	{
		languageOptions: {
			globals: {
				...globals.browser,
				...globals.node,
			},
		},
	},
	{
		rules: {
			'no-console': ['warn', { allow: ['warn', 'error'] }],
		},
	},
	{
		ignores: ['dist/', '.svelte-kit/', 'node_modules/', 'target/'],
	},
];
