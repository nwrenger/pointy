<script lang="ts">
	import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
	import {
		run_extension,
		read_to_string,
		type ExtensionInfo,
		update_extensions,
		update_config
	} from '$lib/api';
	import { BaseDirectory, watchImmediate, type WatchEvent } from '@tauri-apps/plugin-fs';

	let items: ExtensionInfo[] = $state([]);
	let debounceTimeout1: NodeJS.Timeout | null = null;

	async function populateItems(event: WatchEvent | undefined) {
		if (debounceTimeout1) clearTimeout(debounceTimeout1);
		debounceTimeout1 = setTimeout(async () => {
			let all_items = await update_extensions();
			items = all_items.filter((i) => i.enabled);
			debounceTimeout1 = null;
		}, 100);
	}

	// Once on start
	populateItems(undefined);

	// Update items on directory changes
	watchImmediate('extensions', populateItems, {
		baseDir: BaseDirectory.AppData,
		recursive: true
	});

	let debounceTimeout2: NodeJS.Timeout | null = null;

	async function populateChanges() {
		if (debounceTimeout2) clearTimeout(debounceTimeout2);
		debounceTimeout2 = setTimeout(async () => {
			console.log('Config changes getting applied!');
			await update_config();
			debounceTimeout2 = null;
		}, 100);
	}

	// Update config on directory changes
	watchImmediate('config.json', populateChanges, {
		baseDir: BaseDirectory.AppData,
		recursive: false
	});

	const buttonSize = 33;
	const current_window = getCurrentWindow();

	let radius = $derived(items.length * 12);
	let angleStep = $derived(360 / items.length);
	let size = $derived(2 * radius + buttonSize + 2);

	$effect(() => {
		current_window.setSize(new LogicalSize(size, size));
	});
	current_window.listen('select-option', async () => {
		if (current_option) {
			await run_extension(current_option);
			current_option = undefined;
		}
	});

	const timeout_duration = 150;
	let active_timeout: NodeJS.Timeout | undefined = $state();
	let current_option: string | undefined = $state();

	function mouseouseEnter(itemAction: string) {
		if (active_timeout) {
			clearTimeout(active_timeout);
		}
		active_timeout = setTimeout(() => {
			current_option = itemAction;
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
										{current_option === item.abbreveation
					? 'outline preset-tonal-success duration-75'
					: 'preset-tonal-surface duration-0'}"
				aria-label={item.name}
				title={item.description}
				onfocus={() => {}}
				onmouseover={() => mouseouseEnter(item.abbreveation)}
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
				{#await read_to_string(item.icon_path) then contents}
					<span class="cursor-pointer">
						{@html contents}
					</span>
				{/await}
			</button>
		{/each}
	</div>
</div>
