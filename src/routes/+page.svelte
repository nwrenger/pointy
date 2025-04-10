<script lang="ts">
	import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
	import { invoke } from '@tauri-apps/api/core';
	import { items } from './../lib';

	const radius = items.length * 12;
	const angleStep = 360 / items.length;

	let current_option: string | undefined = $state();

	getCurrentWindow().listen('select-option', () => {
		if (current_option) {
			invoke(current_option);
		}
	});

	$effect(() => {
		let s = radius * 2 + 2 * 33;
		getCurrentWindow().setSize(new LogicalSize(s, s));
	});
</script>

<div class="flex items-center justify-center h-full">
	<div class="relative">
		{#each items as item, i}
			{@const angle = angleStep * i - 90}
			<button
				class="absolute btn-icon cursor-pointer {current_option === item.action
					? 'outline preset-tonal-success'
					: 'preset-tonal-surface'}"
				title={item.descrption}
				onfocus={() => {}}
				onmouseover={() => (current_option = item.action)}
				onmouseleave={() => (current_option = undefined)}
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
