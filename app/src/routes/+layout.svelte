<script lang="ts">
	import { Toaster } from '@skeletonlabs/skeleton-svelte';
	import { overrideItemIdKeyNameBeforeInitialisingDndZones } from 'svelte-dnd-action';
	import { toaster } from '$lib/toaster';
	import '../app.css';

	let { children } = $props();

	overrideItemIdKeyNameBeforeInitialisingDndZones('icon_path');

	// Disable context menu
	function onContextMenu(e: MouseEvent) {
		e.preventDefault();
	}
</script>

<Toaster {toaster}></Toaster>

<svelte:document oncontextmenu={onContextMenu} />

{@render children()}
