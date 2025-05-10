<script lang="ts">
	import api from '$lib/api';
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { Circle, Download } from 'lucide-svelte';
	import { handle_promise } from '$lib/toaster';

	let { already_installed = $bindable() }: { already_installed: api.InstalledExtensionInfo[] } =
		$props();
	let open = $state(false);
	let downloading: Record<string, boolean> = $state({});

	let extensions: api.AvailableExtension[] | null = $state(null);
	async function fetchExtensions() {
		extensions = null;
		extensions = await handle_promise(api.fetch_online_extensions());
	}

	$effect(() => {
		if (open) fetchExtensions();
	});

	let needle = $state('');
	let showOptions: 'all' | 'installed' | 'not-installed' = $state('all');
	let filtered: api.AvailableExtension[] | null = $derived.by(filter);

	function filter() {
		console.log(extensions);
		if (extensions != null) {
			const lowerNeedle = needle.toLowerCase();

			function filter_installed(ext: api.AvailableExtension): boolean {
				const isInstalled = already_installed.some((e) => e.manifest.id === ext.id);
				if (showOptions === 'all') {
					return true;
				} else if (showOptions === 'installed' && isInstalled) {
					return true;
				} else if (showOptions === 'not-installed' && !isInstalled) {
					return true;
				}
				return false;
			}

			// If no search term, just filter by installed flags and preserve order
			if (!needle) {
				return extensions.filter((ext) => filter_installed(ext));
			}

			type Ranked = { ext: api.AvailableExtension; priority: number; idx: number };

			const ranked: Ranked[] = extensions.map((ext, idx) => {
				const name = ext.name.toLowerCase();
				const desc = ext.description.toLowerCase();
				const author = ext.author.toLowerCase();

				let priority: number;

				if (name === lowerNeedle) {
					// 1) exact match of title
					priority = 1;
				} else if (name.startsWith(lowerNeedle)) {
					// 2) starting match of title
					priority = 2;
				} else if (name.includes(lowerNeedle)) {
					// 3) matching any part of the title
					priority = 3;
				} else if (desc.includes(lowerNeedle)) {
					// 4) matching any part of the description
					priority = 4;
				} else if (author.includes(lowerNeedle)) {
					// 5) matching any part of the author
					priority = 5;
				} else {
					// no match → drop
					priority = Number.POSITIVE_INFINITY;
				}

				return { ext, priority, idx };
			});

			return (
				ranked
					// drop non‐matches
					.filter(({ priority }) => priority !== Number.POSITIVE_INFINITY)
					// filter by installed/not‐installed flags
					.filter(({ ext }) => filter_installed(ext))
					// sort by priority, then original index to stabilize ties
					.sort((a, b) => a.priority - b.priority || a.idx - b.idx)
					// return only the manifests
					.map(({ ext }) => ext)
			);
		} else {
			return null;
		}
	}

	async function download(ext: api.AvailableExtension) {
		const id = ext.id;
		downloading[id] = true;
		try {
			const installed = await handle_promise(
				api.download_and_install_extension(id, ext.latest_url)
			);
			already_installed.push(installed);
		} finally {
			downloading[id] = false;
		}
	}

	function modalClose() {
		open = false;
	}
</script>

<Modal
	{open}
	onOpenChange={(e) => (open = e.open)}
	triggerBase=""
	contentBase="card preset-tonal p-4 gap-2 shadow-xl max-w-screen-sm h-full w-full grid grid-rows-[35px_1fr_35px] min-h-0 z-[9]"
	backdropClasses="backdrop-blur-sm rounded"
>
	{#snippet trigger()}
		<button class="btn-icon preset-filled" title="Download - Not yet implemented">
			<Download class="size-4" />
		</button>
	{/snippet}
	{#snippet content()}
		<header class="input-group grid-cols-[1fr_auto]">
			<input bind:value={needle} class="ig-input rounded-l" type="text" placeholder="Search..." />
			<select bind:value={showOptions} class="ig-select rounded-r">
				<option selected value="all">View All</option>
				<option value="installed">Installed</option>
				<option value="not-installed">Not Installed</option>
			</select>
		</header>
		<article class="overflow-y-scroll min-h-0 space-y-4">
			{#if filtered == null}
				<p class="opacity-70 italic p-2">Fetching extension metadata...</p>
			{:else}
				{#each filtered as extension_manifest (extension_manifest.id)}
					{@const installed = !!already_installed.find(
						(e) => e.manifest.id == extension_manifest.id
					)}
					<li class="p-4 card preset-tonal grid sm:grid-cols-[1fr_auto] gap-4 items-center">
						<div>
							<h5 class="h5">{extension_manifest.name}</h5>
							<p class="text-sm opacity-70 mt-1">{extension_manifest.description}</p>
							<p class="text-xs mt-2">By {extension_manifest.author}</p>
						</div>
						<button
							class="btn preset-filled-success-500"
							disabled={installed || downloading[extension_manifest.id]}
							title={downloading[extension_manifest.id]
								? 'Downloading…'
								: installed
									? 'Already Installed'
									: 'Download'}
							onclick={() => download(extension_manifest)}
						>
							{#if downloading[extension_manifest.id]}
								<Circle class="animate-ring-indeterminate size-4" /> Downloading...
							{:else if installed}
								Installed
							{:else}
								Download
							{/if}
						</button>
					</li>
				{:else}
					<p class="opacity-70 px-1 italic">Search query returned no results...</p>
				{/each}
			{/if}
		</article>
		<footer class="flex justify-end">
			<button type="button" class="btn preset-tonal" onclick={modalClose}>Close</button>
		</footer>
	{/snippet}
</Modal>
