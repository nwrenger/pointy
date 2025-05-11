<script lang="ts">
	import { Switch, Toaster } from '@skeletonlabs/skeleton-svelte';
	import { flip } from 'svelte/animate';
	import { dragHandle, dragHandleZone, type DndEvent } from 'svelte-dnd-action';
	import { AlignJustify, Circle, Power, PowerOff, RefreshCw, Trash2 } from 'lucide-svelte';
	import { getCurrentWindow, Window } from '@tauri-apps/api/window';
	import { areObjectsEqual, deepClone } from '$lib/utils';
	import ExtensionsModal from './ExtensionsModal.svelte';
	import { handle_promise, toaster } from '$lib/toaster';
	import api from '$lib/api';

	const defaultFlipDurationMs = 300;
	const current_window = getCurrentWindow();
	let flipDurationMs = $state(defaultFlipDurationMs);

	let config: api.Config | undefined = $state();
	let edited_config: api.Config | undefined = $state();
	let extensions: api.InstalledExtensionInfo[] = $state([]);
	let edited_extensions: api.InstalledExtensionInfo[] = $state([]);

	// Initialize config
	async function init() {
		config = await handle_promise(api.get_config());
	}
	init();

	current_window.listen('open-settings', async () => {
		edited_config = deepClone(config);
		extensions = await handle_promise(api.get_installed_extensions());
		edited_extensions = deepClone(extensions);
		// Re-enable animations after drag
		setTimeout(() => (flipDurationMs = defaultFlipDurationMs), defaultFlipDurationMs);
	});

	// Wait for changes of the extensions but also preserving the order and enabled attributes
	async function wait_changes() {
		let main_window = await Window.getByLabel('main');
		if (main_window) {
			main_window.listen('update-extensions', ({ payload }) => {
				let payload_typed = payload as api.InstalledExtensionInfo[];

				const updateMap = new Map<string, api.InstalledExtensionInfo>();
				for (const e of payload_typed) {
					updateMap.set(e.manifest.id, e);
				}

				extensions = extensions.map((old) => {
					const updated = updateMap.get(old.manifest.id);
					return updated
						? {
								manifest: updated.manifest,
								icon_path: updated.icon_path,
								enabled: old.enabled
							}
						: old;
				});

				edited_extensions = edited_extensions.map((old) => {
					const updated = updateMap.get(old.manifest.id);
					return updated
						? {
								manifest: updated.manifest,
								icon_path: updated.icon_path,
								enabled: old.enabled
							}
						: old;
				});
			});
		}
	}
	wait_changes();

	let updating_app = $state(false);
	let updating_extensions = $state(false);
	let deleting: Record<string, boolean> = $state({});

	function handleDndFinalize(event: CustomEvent<DndEvent>) {
		const { items: newOrder } = event.detail;
		edited_extensions = newOrder as any[];
	}

	async function app_update() {
		updating_app = true;
		try {
			await handle_promise(api.update_app());
		} finally {
			updating_app = false;
		}
	}

	async function extensions_update() {
		updating_extensions = true;
		try {
			await handle_promise(api.update_extensions());
		} finally {
			updating_extensions = false;
		}
	}

	async function remove(id: string) {
		deleting[id] = true;
		try {
			await handle_promise(api.delete_extension(id));
			// Filter both because when deleting extensions it changes also the config like this
			extensions = extensions.filter((e) => e.manifest.id !== id);
			edited_extensions = edited_extensions.filter((e) => e.manifest.id !== id);
		} finally {
			deleting[id] = false;
		}
	}

	async function cancel() {
		// Disable animations
		flipDurationMs = 0;
		current_window.hide();
	}

	async function apply() {
		if (!edited_config) return;

		edited_config.enabled = edited_extensions.filter((e) => e.enabled).map((e) => e.manifest.id);
		edited_config.ordered = edited_extensions.map((e) => e.manifest.id);

		config = await handle_promise(api.change_config(edited_config));

		current_window.hide();
	}
</script>

<Toaster {toaster}></Toaster>

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
			<p>Version {#await handle_promise(api.get_version()) then version}{version}{/await}</p>
			<button
				class="btn-icon preset-filled"
				disabled={updating_app}
				title={updating_app ? 'Updating...' : 'Check for Updates'}
				onclick={app_update}
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
				checked={edited_config?.autolaunch}
				onCheckedChange={(e) => {
					if (edited_config) edited_config.autolaunch = e.checked;
				}}
			></Switch>
		</div>

		<hr class="hr" />

		<div class="flex justify-between items-center gap-4">
			<p>Shortcut</p>
			<input
				value={edited_config?.shortcut}
				oninput={(e) => {
					let target = e.target as HTMLInputElement;
					if (edited_config) edited_config.shortcut = target.value;
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
					<ExtensionsModal bind:already_installed={edited_extensions} />
					<button
						class="btn-icon preset-filled"
						disabled={updating_extensions}
						title={updating_extensions ? 'Updating...' : 'Check all for Updates'}
						onclick={extensions_update}
					>
						{#if updating_extensions}
							<Circle class="animate-ring-indeterminate size-4" />
						{:else}
							<RefreshCw class="size-4" />
						{/if}
					</button>
				</div>
			</div>

			{#if edited_extensions.length != 0}
				<section
					use:dragHandleZone={{ items: edited_extensions, flipDurationMs }}
					onconsider={handleDndFinalize}
					onfinalize={handleDndFinalize}
				>
					{#each edited_extensions as extension (extension.icon_path)}
						<div
							class="flex w-full items-center preset-tonal border-b border-surface-200-800 last:border-0 py-4"
							animate:flip={{ duration: flipDurationMs }}
						>
							<div class="py-2 px-3" use:dragHandle>
								<AlignJustify class="size-4" />
							</div>
							<div class="w-full items-center justify-between grid grid-cols-[auto_85px]">
								<p class="truncate w-full">
									{extension.manifest.name}
								</p>
								<div class="flex items-center space-x-2 pe-3 justify-end">
									<button
										class="btn-icon {extension.enabled ? 'preset-filled' : 'preset-glass-neutral'}"
										title={extension.enabled ? 'Enabled' : 'Disabled'}
										onclick={() => (extension.enabled = !extension.enabled)}
									>
										{#if extension.enabled}
											<Power class="size-4" />
										{:else}
											<PowerOff class="size-4" />
										{/if}
									</button>
									<button
										class="btn-icon box-[none] flex preset-filled-error-500 z-10"
										title={deleting[extension.manifest.id] ? 'Removing…' : 'Remove'}
										disabled={updating_extensions || deleting[extension.manifest.id]}
										onclick={() => remove(extension.manifest.id)}
									>
										{#if deleting[extension.manifest.id]}
											<Circle class="animate-ring-indeterminate size-4" />
										{:else}
											<Trash2 class="size-4 text-destructive" />
										{/if}
									</button>
								</div>
							</div>
						</div>
					{/each}
				</section>
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
			disabled={areObjectsEqual(config, edited_config) &&
				areObjectsEqual(extensions, edited_extensions)}
			onclick={apply}>Apply</button
		>
	</div>
</div>
