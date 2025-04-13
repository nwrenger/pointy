<script lang="ts">
	import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
	import {
		activated_extensions,
		run_extension,
		read_to_string,
		type ExtensionInfo
	} from '$lib/api';
	import { BaseDirectory, watchImmediate } from '@tauri-apps/plugin-fs';

	let items: ExtensionInfo[] = $state([]);

	async function populateItems() {
		items = await activated_extensions();
		console.log('asd');
	}

	// Once on start
	populateItems();

	// Update items on directory changes
	watchImmediate('plugins', populateItems, {
		baseDir: BaseDirectory.AppData,
		recursive: true
	});

	const padding = 5;
	const iconSize = 33;

	let radius = $derived(items.length * 12);
	let angleStep = $derived(360 / items.length);

	let size = $derived(2 * (radius + iconSize / 2) + 2 * padding);
	let current_window = getCurrentWindow();

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
