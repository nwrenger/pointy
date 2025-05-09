<script lang="ts">
	import { Switch, Tooltip } from '@skeletonlabs/skeleton-svelte';
	import { flip } from 'svelte/animate';
	import { dragHandle, dragHandleZone, type DndEvent } from 'svelte-dnd-action';
	import { AlignJustify, Circle, Info, RefreshCw, Trash2 } from 'lucide-svelte';
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
	import ExtensionsModal from './ExtensionsModal.svelte';

	const defaultFlipDurationMs = 300;
	const current_window = getCurrentWindow();
	let flipDurationMs = $state(defaultFlipDurationMs);

	let config: Config | undefined = $state();
	let new_config: Config | undefined = $state();
	let extensions: ExtensionInfo[] = $state([]);
	let new_extensions: ExtensionInfo[] = $state([]);
	let error: unknown | undefined = $state();

	// Initialize config
	async function init() {
		config = await get_config();
	}
	init();

	current_window.listen('open-settings', async () => {
		try {
			new_config = deepClone(config);
			extensions = await get_extensions();
			new_extensions = deepClone(extensions);
			// Re-enable animations after drag
			setTimeout(() => (flipDurationMs = defaultFlipDurationMs), defaultFlipDurationMs);
			// Remove any errors, everything worked!
			error = undefined;
		} catch (e) {
			error = e;
			throw e;
		}
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
		} finally {
			updating_app = false;
		}
	}

	async function init_extensions_update() {
		updating_extensions = true;
		try {
			await update_extensions();
		} finally {
			updating_extensions = false;
		}
	}

	async function cancel() {
		// Disable animations
		flipDurationMs = 0;
		current_window.hide();
	}

	async function apply() {
		if (new_config) {
			new_config.enabled = new_extensions.filter((e) => e.enabled).map((e) => e.manifest.id);
			new_config.ordered = new_extensions.map((e) => e.manifest.id);

			config = await change_config(new_config);
		}

		current_window.hide();
	}
</script>

<div class="h-full preset-glass-neutral rounded grid grid-rows-[32px_auto_56px]">
	<!-- Header -->
	<div data-tauri-drag-region class="flex items-center justify-between px-3">
		<h4 data-tauri-drag-region class="h4">Settings</h4>
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
		>
			<circle data-tauri-drag-region cx="12" cy="9" r="1" />
			<circle data-tauri-drag-region cx="19" cy="9" r="1" />
			<circle data-tauri-drag-region cx="5" cy="9" r="1" />
			<circle data-tauri-drag-region cx="12" cy="15" r="1" />
			<circle data-tauri-drag-region cx="19" cy="15" r="1" />
			<circle data-tauri-drag-region cx="5" cy="15" r="1" />
		</svg>
	</div>

	<!-- Content -->
	<div class="px-3 space-y-4 overflow-y-scroll h-full pt-4 pb-2">
		<div class="flex justify-between items-center gap-4">
			<p>Version {#await get_version() then version}{version}{/await}</p>
			<button
				class="btn-icon preset-filled"
				disabled={updating_app}
				title={updating_app ? 'Updating...' : 'Check for Updates'}
				onclick={init_app_update}
			>
				{#if updating_app}
					<Circle class="animate-ring-indeterminate size-4" />
				{:else}
					<RefreshCw class="size-4" />
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
				placeholder="Shortcut..."
			/>
		</div>

		<hr class="hr" />

		<div class="space-y-2">
			<div class="flex justify-between items-center mb-3">
				<div class="flex justify-center space-x-2 items-center">
					<h5 class="h5">Extensions</h5>

					<p class="opacity-70"></p>
				</div>
				<div class="flex items-center space-x-2">
					<ExtensionsModal bind:already_downloaded={new_extensions} />
					<button
						class="btn-icon preset-filled"
						disabled={updating_extensions}
						title={updating_extensions ? 'Updating...' : 'Check all for Updates'}
						onclick={init_extensions_update}
					>
						{#if updating_extensions}
							<Circle class="animate-ring-indeterminate size-4" />
						{:else}
							<RefreshCw class="size-4" />
						{/if}
					</button>
				</div>
			</div>

			{#if new_extensions.length != 0}
				<section
					use:dragHandleZone={{ items: new_extensions, flipDurationMs }}
					onconsider={handleDndFinalize}
					onfinalize={handleDndFinalize}
				>
					{#each new_extensions as extension (extension.icon_path)}
						<div
							class="flex w-full items-center space-x-2 preset-tonal border-b border-surface-200-800 last:border-0 py-4"
							animate:flip={{ duration: flipDurationMs }}
						>
							<div class="ps-3 p-1" use:dragHandle>
								<AlignJustify class="size-4" />
							</div>
							<div class="w-full items-center justify-between grid grid-cols-[auto_85px]">
								<p class="truncate w-full">
									{extension.manifest.name}
								</p>
								<div class="flex items-center space-x-4 pe-3 justify-end">
									<input
										class="checkbox"
										type="checkbox"
										checked={extension.enabled}
										oninput={(e) => (extension.enabled = (e.target as HTMLInputElement).checked)}
									/>
									<button
										class="btn-icon box-[none] flex preset-filled-error-500 z-10"
										title="Remove - Not yet implemented"
									>
										<Trash2 class="size-4 text-destructive" />
									</button>
								</div>
							</div>
						</div>
					{/each}
				</section>
			{:else if error}
				<p class="text-error-300">An Error occured: {error}</p>
			{:else}
				<p class="opacity-70 italic">No extensions downloaded...</p>
			{/if}
		</div>
	</div>

	<!-- Footer -->
	<div class="flex justify-between items-center px-3 space-x-2">
		<button class="btn preset-filled-error-50-950" onclick={cancel}>Close</button>
		<button
			class="btn preset-filled"
			disabled={areObjectsEqual(config, new_config) && areObjectsEqual(extensions, new_extensions)}
			onclick={apply}>Apply</button
		>
	</div>
</div>
