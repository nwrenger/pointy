import { invoke } from '@tauri-apps/api/core';

export interface ExtensionManifest {
	name: string;
	display_name: string;
	version: string;
	description: string;
	latest_url: string;
}

export interface ExtensionInfo {
	manifest: ExtensionManifest;
	icon_path: string;
	enabled: boolean;
}

export interface Config {
	autolaunch: boolean;
	shortcut: string;
	enabled: string[];
	ordered: string[];
}

export async function get_version(): Promise<string> {
	return await invoke('get_version');
}

export async function get_extensions(): Promise<ExtensionInfo[]> {
	return await invoke('get_extensions');
}

export async function run_extension(extension_name: string): Promise<void> {
	return await invoke('run_extension', {
		extensionName: extension_name
	});
}

export async function update_app(): Promise<void> {
	return await invoke('update_app');
}

export async function update_extensions(): Promise<void> {
	return await invoke('update_extensions');
}

export async function get_config(): Promise<Config> {
	return await invoke('get_config');
}

export async function change_config(new_config: Config): Promise<Config> {
	return await invoke('change_config', { newConfig: new_config });
}

export async function read_to_string(path: string): Promise<string> {
	return await invoke('read_to_string', { path: path });
}
