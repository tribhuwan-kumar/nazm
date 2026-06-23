<script lang="ts">
  import { formatSpeed } from "$lib/utils";
  import { systemState } from '$lib/system';
  import { Badge } from "$lib/components/ui/badge";
  import { HugeiconsIcon } from "@hugeicons/svelte";
  import { Button } from "$lib/components/ui/button";
	import AddDl from '$lib/components/AddDl.svelte';
  import * as Dialog from "$lib/components/ui/dialog";
  import { Label } from "$lib/components/ui/label/index.js";
  import { Checkbox } from "$lib/components/ui/checkbox/index.js";
	import * as SidebarUI from "$lib/components/ui/sidebar/index.js";
  import {
    aria2,
		aria2Store,
  } from '$lib/aria2/client.svelte';
  import {
    selectedGids,
    clearSelection,
  } from '$lib/selection';
	import {
    PlayIcon,
    ListXIcon,
    PauseIcon,
    PlusSignIcon,
    Search02Icon,
    Cancel02Icon,
    Delete02Icon,
    ArrowUpDoubleIcon,
    ArrowDownDoubleIcon,
    CheckmarkBadge03Icon,
  } from '@hugeicons/core-free-icons';

  let count = $derived($selectedGids.length);
  let globalStats = $derived(aria2Store.global);
  let hasSelection = $derived(count > 0);
  let isAria2Alive = $derived($systemState.status?.aria2Alive);
  let selectedItems = $derived(aria2Store.items.filter(i => $selectedGids.includes(i.gid)));

  let canResume = $derived(selectedItems.some(i => ['paused'].includes(i.status)));
  let canPause = $derived(selectedItems.some(i => ['active', 'waiting'].includes(i.status)));
  let canStop = $derived(selectedItems.some(i => ['active', 'waiting', 'paused', 'error'].includes(i.status)));

  let deleteDialogOpen = $state(false);
  let deleteWithFiles = $state(false);
  let addDialogOpen = $state(false);
  let gidsToDelete = $state<string[]>([]);


  function handleAdd() {
    addDialogOpen = true;
  }

  // Only `paused` and `stopped` gets resumed
  async function handleResume() {
    const targets = selectedItems.filter(i => ['paused', 'stopped'].includes(i.status));
    await Promise.all(targets.map(i => aria2.resume(i.gid)));
    clearSelection();
  }

	// Only `active` and `waiting` can be paused,
	// `waiting` doesn't emits the notification event on being paused, had to it handle manually
  async function handlePause() {
    const targets = selectedItems.filter(i => ['active', 'waiting'].includes(i.status));
    await Promise.all(targets.map(i => aria2.pause(i.gid)));
    clearSelection();
  }

  // Stopping -> removing from queue, freeing aria2 memory
  async function handleStop() {
    const targets = selectedItems.filter(i => ['active', 'waiting', 'paused', 'error'].includes(i.status));
    await Promise.all(targets.map(i => aria2.stop(i.gid)));
    clearSelection();
  }

  /// Remove -> Delete
  function promptDelete() {
    if ($selectedGids.length === 0) return;
    gidsToDelete = $selectedGids;
    deleteWithFiles = false;
    deleteDialogOpen = true;
  }

  function confirmDelete() {
    aria2.delete(gidsToDelete, deleteWithFiles);
    deleteDialogOpen = false;
    clearSelection();
  }

  function handleSearch() {

  }

	$effect(() => {
		console.log("Aria2 alive:", isAria2Alive);
	});
</script>


<div class="bg-background/95 backdrop-blur  supports-backdrop-filter:bg-background/60 sticky top-0 z-10 w-full">

	<div class="inline-flex flex-row">
		<SidebarUI.Trigger class="m-2 text-muted-foreground bg-background/95 backdrop-blur supports-backdrop-filter:bg-background/60 sticky top-0 cursor-pointer" />

		<div class="hidden sm:inline-flex justify-start float-left md:h-12 items-center md:px-4 gap-2">
			<div class="flex items-center gap-4 text-sm text-muted-foreground">
				{#if globalStats.downloadSpeed !== "0" || globalStats.uploadSpeed !== "0"}
					<Badge
						variant="outline"
						title="Global downloading speed"
						class="gap-1 px-2 py-3.5 min-w-32 font-mono cursor-pointer text-xs animate-in fade-in zoom-in duration-200"
					>
						{#if globalStats.downloadSpeed !== "0"}
							<HugeiconsIcon
								icon={ArrowDownDoubleIcon}
								size={24}
								color="currentColor"
								strokeWidth={2}
								class="w-8! h-8!"
							/>
							{formatSpeed(globalStats.downloadSpeed)}
						{/if}
						{#if globalStats.uploadSpeed !== "0"}
							<div class="w-px h-5 mx-2 bg-border"></div>
								<HugeiconsIcon
									icon={ArrowUpDoubleIcon}
									size={24}
									color="currentColor"
									strokeWidth={1.5}
								/>
							{formatSpeed(globalStats.uploadSpeed)}
						{/if}
					</Badge>
				{/if}
			</div>
		</div>
	</div>

  <div class="inline-flex justify-end float-right h-12 items-center px-4 gap-2">
    <div class="hidden sm:flex items-center text-sm text-muted-foreground">
      {#if hasSelection}
        <div class="flex items-center text-primary/90 gap-2 border border-muted-foreground/20 px-2 py-1 rounded-full animate-in fade-in zoom-in duration-200">
          <HugeiconsIcon
            icon={CheckmarkBadge03Icon}
            size={18}
            color="currentColor"
            strokeWidth={1.5}
						class="md:w-4.5 w-4"
          />
          <span class="font-mono max-xs:text-[12px] text-xs">{count} Selected</span>
          <button
            onclick={clearSelection}
            class="md:ml-1 hover:bg-primary/20 rounded-full p-0.5 cursor-pointer"
            title="Cancel selection"
          >
            <HugeiconsIcon
              icon={Cancel02Icon}
              color="currentColor"
              strokeWidth={1.5}
							class="md:w-4.5 w-4"
            />
          </button>
        </div>
        <div class="w-px h-6 mx-2 bg-border"></div>
      {/if}
    </div>
    <Button
      onclick={handleSearch}
      disabled={!isAria2Alive}
      class="cursor-pointer! disabled:cursor-not-allowed!"
      size="icon"
      variant="ghost"
      title="Search"
    >
      <HugeiconsIcon
        icon={Search02Icon}
        class="text-muted-foreground w-4! h-4!"
        strokeWidth={2}
      />
    </Button>
    <div class="w-px h-6 mx-2 bg-border"></div>
    <Button
		disabled={!isAria2Alive}
      onclick={handleAdd}
      class="cursor-pointer disabled:cursor-not-allowed!"
      size="icon"
      variant="ghost"
      title="Add downloads"
    >
      <HugeiconsIcon
        icon={PlusSignIcon}
        class="text-muted-foreground w-4! h-4!"
        strokeWidth={2}
      />
    </Button>
    <AddDl
      bind:open={addDialogOpen}
      onclose={() => addDialogOpen = false}
    />
		<div class="w-px h-6 mx-2 bg-border hidden md:block"></div>
		<!-- md screen action buttons -->
    <div class="items-center gap-1 hidden md:flex">
      <Button
        variant="ghost"
        size="icon"
				class="disabled:cursor-not-allowed! cursor-pointer"
        disabled={!hasSelection || !isAria2Alive || !canResume}
        onclick={handleResume}
        title="Resume selected"
      >
        <HugeiconsIcon
          icon={PlayIcon}
          class="text-muted-foreground w-4! h-4!"
          strokeWidth={2}
          size={24}
        />
      </Button>

      <Button
        variant="ghost"
        size="icon"
        disabled={!hasSelection || !isAria2Alive || !canPause}
				class="disabled:cursor-not-allowed! cursor-pointer"
        onclick={handlePause}
        title="Pause selected"
      >
        <HugeiconsIcon
          icon={PauseIcon}
          class="text-muted-foreground w-4! h-4!"
          strokeWidth={2}
          size={24}
        />
      </Button>

      <Button
        variant="ghost"
        size="icon"
        disabled={!hasSelection || !isAria2Alive || !canStop}
        class="text-muted-foreground cursor-pointer! disabled:cursor-not-allowed!"
        onclick={handleStop}
        title="Stop from downloading"
      >
        <HugeiconsIcon
          icon={ListXIcon}
          class="w-4! h-4!"
          strokeWidth={2}
          size={24}
        />
      </Button>

      <Button
        size="icon"
        variant="ghost"
        class="text-muted-foreground hover:text-destructive! cursor-pointer! disabled:cursor-not-allowed! hover:bg-destructive/10!"
        disabled={!hasSelection || !isAria2Alive}
        onclick={promptDelete}
        title="Delete selected"
      >
        <HugeiconsIcon
          size={24}
          strokeWidth={2}
          icon={Delete02Icon}
          class="w-4! h-4!"
        />
      </Button>
    </div>
  </div>
</div>

<Dialog.Root bind:open={deleteDialogOpen}>
  <Dialog.Content class="sm:max-w-[500px] sm:min-h-[350px]">
    <Dialog.Header>
      <Dialog.Title>Confirm Delete</Dialog.Title>
      <Dialog.Description>
        Are you sure you want to delete this download?
        <br/> this action cannot be undone.
        <br/><br/>
      </Dialog.Description>
    </Dialog.Header>

    <div class="flex flex-row gap-2">
      <div class="w-4 h-6 mt-1 inline">
        <Checkbox
          class="cursor-pointer"
          id="delete-with-files"
          bind:checked={deleteWithFiles}
        />
      </div>
      <div class="inline">
        <Label
          for="delete-with-files"
          class="cursor-pointer"
        >
          Delete files too!!??
        </Label>
				<br/>
        <span class="text-[14px] font-normal text-muted-foreground">be careful, it'll delete their actual content from server!!</span>
      </div>
    </div>

    <Dialog.Footer
      class="mt-2"
    >
      <Button
        class="cursor-pointer"
        variant="outline"
        onclick={() => deleteDialogOpen = false}
      >
        Cancel
      </Button>
      <Button
        class="min-w-24 cursor-pointer"
        variant="destructive"
        onclick={confirmDelete}
      >
        Delete
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

