<script lang="ts">
	import type { ExtensionInfo } from '$lib/api';
	// TODO: Fetching Data from repo
	import { extensions_from_online } from '$lib/utils';
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { Download, Settings } from 'lucide-svelte';

	let { already_downloaded = $bindable() }: { already_downloaded: ExtensionInfo[] } = $props();
	let open = $state(false);

	function modalClose() {
		open = false;
	}
</script>

<Modal
	{open}
	onOpenChange={(e) => (open = e.open)}
	triggerBase=""
	contentBase="card preset-tonal p-4 gap-2 shadow-xl max-w-screen-sm max-h-full grid grid-rows-[35px_1fr_35px] min-h-0"
	backdropClasses="backdrop-blur-sm rounded"
>
	{#snippet trigger()}
		<button class="btn-icon preset-filled" title="Download - Not yet implemented">
			<Download class="size-4" />
		</button>
	{/snippet}
	{#snippet content()}
		<header class="input-group grid-cols-[1fr_auto]">
			<!-- TODO: Real Searching && Sorting -->
			<input class="ig-input rounded-l" type="text" placeholder="Search..." />
			<!-- TODO: Popup -->
			<button class="ig-btn preset-tonal" title="Username already in use.">
				<Settings size={16} />
			</button>
		</header>
		<article class="overflow-y-scroll min-h-0 space-y-4">
			{#each extensions_from_online as extension (extension.id)}
				{@const downloaded = !!already_downloaded.find((e) => e.manifest.id == extension.id)}
				<li class="p-4 card preset-tonal grid sm:grid-cols-[1fr_auto] gap-4 items-center">
					<div>
						<h5 class="h5">{extension.name}</h5>
						<p class="text-sm opacity-70 mt-1">{extension.description}</p>
						<p class="text-xs mt-2">Author: {extension.author}</p>
						<p class="text-xs mt-1">Current Version: {extension.version}</p>
					</div>
					<button class="btn preset-filled-success-500" disabled={downloaded} onclick={() => {}}>
						{#if downloaded}
							Downloaded
						{:else}
							Download
						{/if}
					</button>
				</li>
			{/each}
		</article>
		<footer class="flex justify-end">
			<button type="button" class="btn preset-tonal" onclick={modalClose}>Close</button>
		</footer>
	{/snippet}
</Modal>
