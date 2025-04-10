import { Calculator, Camera, KeyRound, QrCode } from 'lucide-svelte';

export const items = [
	{
		action: 'capture_screenshot',
		descrption:
			'Captures a screenshot of the current monitor by mouse position and copies the result to the clipboard.',
		icon: Camera
	},
	{
		action: 'evaluate_math_equasion',
		descrption: 'Evaluates a math equasion and copies the result to the clipboard.',
		icon: Calculator
	},
	{
		action: 'generate_qrcode',
		descrption: 'Generates a qrcode from copied text and saves it to downloads.',
		icon: QrCode
	},
	{
		action: 'create_secure_password',
		descrption: 'Creates a 12 character long very secure password and copies it to the clipboard.',
		icon: KeyRound
	}
];
