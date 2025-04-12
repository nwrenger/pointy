import { invoke } from '@tauri-apps/api/core';

export interface PluginInfo {
	abbreveation: string;
	name: string;
	description: string;
	icon_path: string;
}

export async function activated_plugins(): Promise<PluginInfo[]> {
	return await invoke('activated_plugins');
}

export async function call_plugin_command(plugin_name: string): Promise<void> {
	return await invoke('call_plugin_command', { pluginName: plugin_name });
}

export async function read_to_string(path: string): Promise<string> {
	return await invoke('read_to_string', { path: path });
}
