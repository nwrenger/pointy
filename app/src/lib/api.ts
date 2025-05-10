import { invoke } from '@tauri-apps/api/core';

namespace api {
	export type Error =
		| { kind: 'PoisonedLock' }
		| { kind: 'Checksum' }
		| { kind: 'NoAssets' }
		| { kind: 'FileSystem'; value: string }
		| { kind: 'LibLoading'; value: string }
		| { kind: 'Conversion'; value: string }
		| { kind: 'Json'; value: string }
		| { kind: 'Reqwest'; value: string }
		| { kind: 'Shortcut'; value: string }
		| { kind: 'Autostart'; value: string }
		| { kind: 'Tauri'; value: string };

	export interface ExtensionManifest {
		id: string;
		name: string;
		author: string;
		version: string;
		description: string;
		latest_url: string;
	}

	export interface AvailableExtension {
		id: string;
		name: string;
		author: string;
		description: string;
		latest_url: string;
	}

	export interface InstalledExtensionInfo {
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

	export async function get_installed_extensions(): Promise<InstalledExtensionInfo[]> {
		return await invoke('get_installed_extensions');
	}

	export async function fetch_online_extensions(): Promise<AvailableExtension[]> {
		return await invoke('fetch_online_extensions');
	}

	export async function run_extension(extension_name: string): Promise<void> {
		return await invoke('run_extension', {
			extensionName: extension_name
		});
	}

	export async function download_and_install_extension(
		id: string,
		latest_url: string
	): Promise<InstalledExtensionInfo> {
		return await invoke('download_and_install_extension', {
			id: id,
			latestUrl: latest_url
		});
	}

	export async function delete_extension(id: string): Promise<void> {
		return await invoke('delete_extension', {
			id: id
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
}

export default api;
