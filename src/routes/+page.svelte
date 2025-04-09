<script lang="ts">
	import { Calculator, Camera, KeyRound, QrCode } from 'lucide-svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { invoke } from '@tauri-apps/api/core';

	getCurrentWindow().listen('select-option', () => {
		if (current_option) {
			invoke(current_option);
		}
	});

	let current_option: string | undefined = $state();

	let items = [
		{
			action: 'capture_screenshot',
			descrption:
				'Captures a screenshot of the current monitor by mouse position and copies the result to the clipboard.',
			icon: Camera
		},
		{
			action: 'evaluate_math_equasion',
			descrption: 'Evaluates a math equasion and copies the result to the clipboard.',
			icon: Calculator
		},
		{
			action: 'generate_qrcode',
			descrption: 'Generates a qrcode from copied text and saves it to downloads.',
			icon: QrCode
		},
		{
			action: 'create_secure_password',
			descrption:
				'Creates a 12 character long very secure password and copies it to the clipboard.',
			icon: KeyRound
		}
	];
</script>

<div class="flex items-center justify-center h-full">
	<div class="relative">
		{#each items as item, i}
			{@const angle = angleStep * )}
			<button
				class="absolute btn cursor-pointer {current_option === item.action
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
