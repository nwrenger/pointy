import { invoke } from '@tauri-apps/api/core';

export interface ExtensionInfo {
	abbreveation: string;
	name: string;
	description: string;
	icon_path: string;
	enabled: boolean;
}

export async function initial_extensions(): Promise<ExtensionInfo[]> {
	return await invoke('initial_extensions');
}

export async function run_extension(extension_name: string): Promise<void> {
	return await invoke('run_extension', { extensionName: extension_name });
}

export async function read_to_string(path: string): Promise<string> {
	return await invoke('read_to_string', { path: path });
}
