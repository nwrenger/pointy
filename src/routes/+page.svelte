<script lang="ts">
	import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
	import { invoke } from '@tauri-apps/api/core';
	import { items } from './../lib';

	const padding = 5;
	const iconSize = 33;

	const radius = items.length * 12;
	const angleStep = 360 / items.length;

	const size = 2 * (radius + iconSize / 2) + 2 * padding;
	const current_window = getCurrentWindow();

	current_window.setSize(new LogicalSize(size, size));
	current_window.listen('select-option', async () => {
		if (current_option) {
			await invoke(current_option);
			current_option = undefined;
		}
	});

	const timeout_duration = 75;
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
										{current_option === item.action
					? 'outline preset-tonal-success duration-75'
					: 'preset-tonal-surface duration-0'}"
				title={item.descrption}
				onfocus={() => {}}
				onmouseover={() => mouseouseEnter(item.action)}
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
				<item.icon class="cursor-pointer" />
			</button>
		{/each}
	</div>
</div>
