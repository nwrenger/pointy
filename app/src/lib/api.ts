import { invoke } from '@tauri-apps/api/core';

export interface ExtensionInfo {
	abbreveation: string;
	name: string;
	description: string;
	icon_path: string;
	enabled: boolean;
}

export async function update_extensions(): Promise<ExtensionInfo[]> {
	return await invoke('update_extensions');
}

export async function update_config(): Promise<void> {
	return await invoke('update_config');
}

export async function info_extensions(): Promise<ExtensionInfo[]> {
	return await invoke('info_extensions');
}

export async function run_extension(extension_name: string): Promise<void> {
	return await invoke('run_extension', { extensionName: extension_name });
}

export async function read_to_string(path: string): Promise<string> {
	return await invoke('read_to_string', { path: path });
}
