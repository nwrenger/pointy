import type api from './api';
import { createToaster } from '@skeletonlabs/skeleton-svelte';
export const toaster = createToaster({ placement: 'bottom-end' });

/** Gets `T` of `api.Result<T>` if no error occurred otherwise displays the error via a toast and throws the error */
export async function handle_promise<T>(
	promise: Promise<T>,
	on_error?: () => void
): Promise<T> | never {
	try {
		return await promise;
	} catch (e: unknown) {
		let error = e as api.Error;

		error_toast(error_msg(error));

		if (on_error) on_error();

		throw error.kind;
	}
}

/** Displays a error toast with improved configuration */
export function error_toast(message: string) {
	toaster.error({ title: 'Error', description: message });
}

/** Server Error translations */
function error_msg(error: api.Error): string {
	switch (error.kind) {
		case 'PoisonedLock':
			return 'Fatal Backend Error: An internal lock was poisoned.';
		case 'Checksum':
			return 'The checksum verification failed.';
		case 'NoAssets':
			return `No assets found for this platform.`;
		case 'FileSystem':
			return `File System Error: ${error.value}.`;
		case 'LibLoading':
			return `Library Loading Error ${error.value}.`;
		case 'Conversion':
			return `Conversion Error: ${error.value}.`;
		case 'Json':
			return `JSON Serialization/Deserialization Error: ${error.value}.`;
		case 'Reqwest':
			return `Network Request Error: ${error.value}.`;
		case 'Tauri':
			return `Tauri Runtime Error: ${error.value}`;
		case 'Shortcut':
			return `Global Shortcut Error: ${error.value}`;
		case 'Autostart':
			return `Autostart Configuration Error: ${error.value}`;
		default:
			return 'An unknown Error has occurred. Try reopening the app!';
	}
}
