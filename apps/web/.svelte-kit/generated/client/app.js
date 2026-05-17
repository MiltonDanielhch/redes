export { matchers } from './matchers.js';

export const nodes = [
	() => import('./nodes/0'),
	() => import('./nodes/1'),
	() => import('./nodes/2'),
	() => import('./nodes/3'),
	() => import('./nodes/4'),
	() => import('./nodes/5'),
	() => import('./nodes/6'),
	() => import('./nodes/7'),
	() => import('./nodes/8'),
	() => import('./nodes/9'),
	() => import('./nodes/10'),
	() => import('./nodes/11'),
	() => import('./nodes/12'),
	() => import('./nodes/13'),
	() => import('./nodes/14'),
	() => import('./nodes/15'),
	() => import('./nodes/16')
];

export const server_loads = [];

export const dictionary = {
		"/": [4],
		"/(dashboard)/admin/audit": [7,[3]],
		"/(dashboard)/admin/users": [8,[3]],
		"/(dashboard)/alerts": [9,[3]],
		"/(dashboard)/dashboard": [10,[3]],
		"/(dashboard)/devices": [11,[3]],
		"/(dashboard)/devices/[id]": [12,[3]],
		"/(dashboard)/intrusions": [13,[3]],
		"/(auth)/login": [5,[2]],
		"/(dashboard)/metrics": [14,[3]],
		"/(auth)/register": [6,[2]],
		"/(dashboard)/sedes": [15,[3]],
		"/(dashboard)/topology": [16,[3]]
	};

export const hooks = {
	handleError: (({ error }) => { console.error(error) }),
	
	reroute: (() => {}),
	transport: {}
};

export const decoders = Object.fromEntries(Object.entries(hooks.transport).map(([k, v]) => [k, v.decode]));
export const encoders = Object.fromEntries(Object.entries(hooks.transport).map(([k, v]) => [k, v.encode]));

export const hash = false;

export const decode = (type, value) => decoders[type](value);

export { default as root } from '../root.js';