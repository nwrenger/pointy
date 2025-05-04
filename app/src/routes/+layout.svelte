<script lang="ts">
	import { overrideItemIdKeyNameBeforeInitialisingDndZones } from 'svelte-dnd-action';
	import '../app.css';

	let { children } = $props();

	overrideItemIdKeyNameBeforeInitialisingDndZones('icon_path');

	// Disable context menu
	function onContextMenu(e: MouseEvent) {
		e.preventDefault();
	}
</script>

<svelte:document oncontextmenu={onContextMenu} />

{@render children()}
