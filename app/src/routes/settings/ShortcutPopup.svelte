<script lang="ts">
	import { isShortcut, keyCodeToKey, keyCombToDisplay, keyCombToShortcut } from '$lib/key_input';
	import { Popover } from '@skeletonlabs/skeleton-svelte';
	import { onMount } from 'svelte';

	let { shortcut = $bindable() } = $props();

	let openState = $state(false);

	let keys: string[] = $state(shortcut.split('+'));
	let recording = $state(false);
	let keyCombination = $derived(keyCombToDisplay(keys));
	let inputRef = $state<HTMLInputElement | null>(null);

	function onKeyDown(e: KeyboardEvent) {
		if (recording) {
			e.preventDefault();
			const newKey = keyCodeToKey(e.code);
			if (keys[keys.length - 1] !== newKey) {
				const newKeys = [...keys, keyCodeToKey(e.code)];
				keys = newKeys;
			}
			if (isShortcut(keys)) {
				recording = false;
			}
		}
	}

	function onKeyUp(e: KeyboardEvent) {
		if (recording) {
			e.preventDefault();
			keys = keys.filter((k) => k !== keyCodeToKey(e.code));
		}
	}

	function submit() {
		shortcut = keyCombToShortcut(keys);
		openState = false;
	}

	function reset(focusAfterDelay: number | undefined = undefined) {
		recording = true;
		keys = [];
		if (focusAfterDelay) setTimeout(() => inputRef?.focus(), focusAfterDelay);
	}
</script>

<Popover
	open={openState}
	onOpenChange={(e) => {
		openState = e.open;
		if (e.open) {
			reset(0);
		}
	}}
	positioning={{ placement: 'bottom', gutter: 5 }}
	triggerBase="btn preset-filled"
	contentBase="card bg-surface-100-900 p-4 space-y-4 w-[260px]"
	arrowBackground="!bg-surface-100 dark:!bg-surface-900"
>
	{#snippet trigger()}{keyCombToDisplay(shortcut.split('+'))}{/snippet}
	{#snippet content()}
		<div class="flex flex-col items-center justify-start space-y-2">
			<!-- svelte-ignore a11y_autofocus -->
			<input
				bind:this={inputRef}
				value={keyCombination}
				onkeydown={onKeyDown}
				onkeyup={onKeyUp}
				onfocus={() => reset()}
				disabled={!recording}
				autofocus
				class="input"
				type="text"
			/>
			<div class="w-full grid gap-2 grid-cols-2">
				<button class="btn preset-filled-error-500" onclick={() => reset(100)} disabled={recording}
					>Clear</button
				>
				<button class="btn preset-filled" disabled={!isShortcut(keys)} onclick={submit}
					>Submit</button
				>
			</div>
		</div>
	{/snippet}
</Popover>
