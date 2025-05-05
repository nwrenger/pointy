<script lang="ts">
	import { Switch } from '@skeletonlabs/skeleton-svelte';
	import { flip } from 'svelte/animate';
	import { dragHandle, dragHandleZone, type DndEvent } from 'svelte-dnd-action';
	import { AlignJustify } from 'lucide-svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import {
		get_config,
		get_extensions,
		get_version,
		change_config,
		update_app,
		update_extensions,
		type Config,
		type ExtensionInfo
	} from '$lib/api';
	import { areObjectsEqual, deepClone } from '$lib/utils';

	const defaultFlipDurationMs = 300;
	const current_window = getCurrentWindow();
	let flipDurationMs = $state(defaultFlipDurationMs);

	let config: Config | undefined = $state();
	let new_config: Config | undefined = $state();
	let extensions: ExtensionInfo[] = $state([]);
	let new_extensions: ExtensionInfo[] = $state([]);

	async function init() {
		config = await get_config();
	}
	init();

	current_window.listen('open-settings', async () => {
		new_config = deepClone(config);
		extensions = await get_extensions();
		new_extensions = deepClone(extensions);
		// Disable Animations for dnd-list
		setTimeout(() => (flipDurationMs = defaultFlipDurationMs), defaultFlipDurationMs);
	});

	let updating_app = $state(false);
	let updating_extensions = $state(false);

	function handleDndFinalize(event: CustomEvent<DndEvent>) {
		const { items: newOrder } = event.detail;
		new_extensions = newOrder as any[];
	}

	async function init_app_update() {
		updating_app = true;
		try {
			await update_app();
			updating_app = false;
		} catch (e) {
			updating_app = false;
			throw e;
		}
	}

	async function init_extensions_update() {
		updating_extensions = true;
		try {
			await update_extensions();
			updating_extensions = false;
		} catch (e) {
			updating_extensions = false;
			throw e;
		}
	}

	async function cancel() {
		flipDurationMs = 0;
		current_window.hide();
	}

	async function apply() {
		if (new_config) {
			new_config.enabled = new_extensions.filter((e) => e.enabled).map((e) => e.manifest.name);
			new_config.ordered = new_extensions.map((e) => e.manifest.name);

			config = await change_config(new_config);
		}

		current_window.hide();
	}
</script>

<div class="h-full preset-glass-neutral rounded grid grid-rows-[24px_auto_48px]">
	<div data-tauri-drag-region class="flex justify-center">
		<svg
			data-tauri-drag-region
			xmlns="http://www.w3.org/2000/svg"
			width="24"
			height="24"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			class="lucide lucide-grip-horizontal-icon lucide-grip-horizontal"
			><circle data-tauri-drag-region cx="12" cy="9" r="1" />
			<circle data-tauri-drag-region cx="19" cy="9" r="1" />
			<circle data-tauri-drag-region cx="5" cy="9" r="1" />
			<circle data-tauri-drag-region cx="12" cy="15" r="1" />
			<circle data-tauri-drag-region cx="19" cy="15" r="1" />
			<circle data-tauri-drag-region cx="5" cy="15" r="1" /></svg
		>
	</div>
	<div class="px-2 space-y-4 overflow-y-scroll h-full pb-2">
		<h3 class="h3">Settings</h3>

		<div class="flex justify-between items-center gap-4">
			<p>Version {#await get_version() then version}{version}{/await}</p>
			<button class="btn preset-filled" disabled={updating_app} onclick={init_app_update}>
				{#if updating_app}
					Updating...
				{:else}
					Check for Updates
				{/if}
			</button>
		</div>

		<hr class="hr" />

		<div class="flex justify-between items-center gap-4">
			<p>Start on Sytem Startup</p>
			<Switch
				name="autolaunch"
				checked={new_config?.autolaunch}
				onCheckedChange={(e) => {
					if (new_config) new_config.autolaunch = e.checked;
				}}
			></Switch>
		</div>

		<hr class="hr" />

		<div class="flex justify-between items-center gap-4">
			<p>Shortcut</p>
			<input
				value={new_config?.shortcut}
				oninput={(e) => {
					let target = e.target as HTMLInputElement;
					if (new_config) new_config.shortcut = target.value;
				}}
				class="input"
				type="text"
				placeholder="Input"
			/>
		</div>

		<hr class="hr" />

		<div class="space-y-2">
			<h4 class="h4">Extensions</h4>
			{#if new_extensions.length != 0}
				<section
					use:dragHandleZone={{ items: new_extensions, flipDurationMs }}
					onconsider={handleDndFinalize}
					onfinalize={handleDndFinalize}
				>
					{#each new_extensions as extension, i (extension.icon_path)}
						<div
							class="flex w-full items-center space-x-2 preset-tonal {i != new_extensions.length - 1
								? 'border-b border-surface-200-800'
								: ''} py-4"
							animate:flip={{ duration: flipDurationMs }}
						>
							<div use:dragHandle>
								<AlignJustify class="ms-3 m-1 size-4" />
							</div>
							<div class="flex w-full items-center justify-between">
								<p>{extension.manifest.display_name}</p>
								<input
									class="checkbox me-4"
									type="checkbox"
									checked={extension.enabled}
									oninput={(e) => {
										let target = e.target as HTMLInputElement;
										if (new_config) extension.enabled = target.checked;
									}}
								/>
							</div>
						</div>
					{/each}
				</section>
			{:else}
				<p class="text-surface-300">No extensions downloaded...</p>
			{/if}

			<button class="btn preset-filled" disabled title="Not currently implemented">Download</button>
			<button
				class="btn preset-filled"
				disabled={updating_extensions}
				onclick={init_extensions_update}
			>
				{#if updating_extensions}
					Updating all...
				{:else}
					Check all for Updates
				{/if}
			</button>
		</div>
	</div>
	<div class="flex items-center justify-between px-2">
		<button class="btn preset-filled-error-50-950" onclick={cancel}>Close</button>
		<button
			class="btn preset-filled"
			disabled={areObjectsEqual(config, new_config) && areObjectsEqual(extensions, new_extensions)}
			onclick={apply}>Apply</button
		>
	</div>
</div>
