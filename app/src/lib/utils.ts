import type { ExtensionManifest } from './api';

/** Function deep-cloning objects with arrays */
export function deepClone(obj: any): any {
	if (obj === null || typeof obj !== 'object') return obj;

	if (Array.isArray(obj)) {
		return obj.map((item) => deepClone(item));
	}

	const clone = {} as any;
	for (const key in obj) {
		clone[key] = deepClone(obj[key]);
	}
	return clone;
}

/** Helper for checking if objects are really equal */
export function areObjectsEqual(obj1: any, obj2: any): boolean {
	if (Array.isArray(obj1) && Array.isArray(obj2)) {
		if (obj1.length !== obj2.length) return false;
		for (let i = 0; i < obj1.length; i++) {
			if (!areObjectsEqual(obj1[i], obj2[i])) return false;
		}
		return true;
	} else if (
		typeof obj1 !== 'object' ||
		typeof obj2 !== 'object' ||
		obj1 === null ||
		obj2 === null
	) {
		return obj1 === obj2;
	}

	const keys1 = Object.keys(obj1);
	const keys2 = Object.keys(obj2);

	if (keys1.length !== keys2.length) {
		return false;
	}

	for (const key of keys1) {
		if (!areObjectsEqual(obj1[key], obj2[key])) {
			return false;
		}
	}

	return true;
}

export const extensions_from_online: ExtensionManifest[] = [
	{
		id: 'capture_screenshot',
		name: 'Capture a Screenshot',
		author: 'Nils Wrenger',
		version: '0.1.0',
		description:
			'Captures a screenshot of the current monitor by mouse position and copies the result to the clipboard.',
		latest_url:
			'https://github.com/nwrenger/pointy/releases/latest/download/capture_screenshot-latest.json'
	},
	{
		id: 'create_secure_password',
		name: 'Creates a secure Password',
		author: 'Nils Wrenger',
		version: '0.1.0',
		description: 'Creates a 12 character long very secure password and copies it to the clipboard.',
		latest_url:
			'https://github.com/nwrenger/pointy/releases/latest/download/create_secure_password-latest.json'
	},
	{
		id: 'generate_qrcode',
		name: 'Generate a QrCode',
		author: 'Nils Wrenger',
		version: '0.1.0',
		description: 'Generates a qrcode from copied text and saves it to the clipboard.',
		latest_url:
			'https://github.com/nwrenger/pointy/releases/latest/download/generate_qrcode-latest.json'
	},
	{
		id: 'math_equasion',
		name: 'Math Equasion Evaluator',
		author: 'Nils Wrenger',
		version: '0.1.0',
		description: 'Evaluates a math equasion and copies the result to the clipboard.',
		latest_url:
			'https://github.com/nwrenger/pointy/releases/latest/download/math_equasion-latest.json'
	},
	{
		id: 'template',
		name: 'Empty',
		author: 'Nils Wrenger',
		version: '0.1.0',
		description: 'Empty.',
		latest_url: 'https://github.com/nwrenger/pointy/releases/latest/download/template-latest.json'
	},
	{
		id: 'text_metadata',
		name: 'Text Metadata',
		author: 'Nils Wrenger',
		version: '0.1.0',
		description:
			'Counts character, words and lines of a copied text and returns the result to the clipboard.',
		latest_url:
			'https://github.com/nwrenger/pointy/releases/latest/download/text_metadata-latest.json'
	}
];
