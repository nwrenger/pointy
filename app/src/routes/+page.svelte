<script lang="ts">
	import api from '$lib/api';
	import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
	import { handle_promise } from '$lib/toaster';
	let current_window = getCurrentWindow();

	let items: api.InstalledExtensionInfo[] = $state([]);
	function setItems(allExtensions: api.InstalledExtensionInfo[]): void {
		items = allExtensions.filter((extension) => extension.enabled);
	}

	async function loadInitialItems(): Promise<void> {
		const e = await handle_promise(api.get_installed_extensions());
		setItems(e);
	}
	loadInitialItems();

	// Update Items on window event
	current_window.listen('update-extensions', ({ payload }) => {
		setItems(payload as api.InstalledExtensionInfo[]);
	});

	const buttonSize = 33;

	let radius = $derived(items.length * 12);
	let angleStep = $derived(360 / items.length);
	let size = $derived(2 * radius + buttonSize + 2);

	current_window.listen('select-option', async () => {
		if (current_option) {
			await handle_promise(api.run_extension(current_option));
			current_option = undefined;
		}
	});

	$effect(() => {
		current_window.setSize(new LogicalSize(size, size));
	});

	const timeout_duration = 150;
	let active_timeout: NodeJS.Timeout | undefined = $state();
	let current_option: string | undefined = $state();

	function mouseouseEnter(id: string) {
		if (active_timeout) {
			clearTimeout(active_timeout);
		}
		active_timeout = setTimeout(() => {
			current_option = id;
			active_timeout = undefined;
		}, timeout_duration);
	}

	function mouseouseLeave() {
		if (active_timeout) {
			clearTimeout(active_timeout);
			active_timeout = undefined;
		}
		current_option = undefined;
	}
</script>

<div class="flex items-center justify-center h-full">
	<div class="relative">
		{#each items as item, i}
			{@const angle = angleStep * i - 90}
			<button
				class="absolute btn-icon cursor-pointer transition-all focus:outline-none
										{current_option === item.manifest.id
					? 'outline preset-tonal-success duration-75'
					: 'preset-tonal-surface duration-0'}"
				aria-label={item.manifest.id}
				title={item.manifest.description}
				onfocus={() => {}}
				onmouseover={() => mouseouseEnter(item.manifest.id)}
				onmouseleave={mouseouseLeave}
				style={`
					top: 50%;
					left: 50%;
					transform: translate(-50%, -50%)
							   rotate(${angle}deg)
							   translateX(${radius}px)
							   rotate(${-angle}deg);
				`}
			>
				{#await handle_promise(api.read_to_string(item.icon_path)) then contents}
					<span class="cursor-pointer">
						{@html contents}
					</span>
				{/await}
			</button>
		{/each}
	</div>
</div>
