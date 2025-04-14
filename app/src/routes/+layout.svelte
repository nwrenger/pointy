<script lang="ts">
	import '../app.css';

	let { children } = $props();

	// Disable context menu
	function onContextMenu(e: MouseEvent) {
		e.preventDefault();
	}
</script>

<svelte:document oncontextmenu={onContextMenu} />

{@render children()}
