<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { FileExclamationPointIcon, Refresh01Icon
  } from '@hugeicons/core-free-icons';

  export let open = false;
  export let onOverwrite: () => void;
  export let onDownloadAgain: () => void;
  export let onCancel: () => void;

  function handleOverwrite() {
    open = false;
    onOverwrite();
  }

  function handleDownloadAgain() {
    open = false;
    onDownloadAgain();
  }

  function handleCancel() {
    open = false;
    onCancel();
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-[425px]">
    <Dialog.Header>
      <Dialog.Title class="flex items-center gap-2 text-amber-500">
        <HugeiconsIcon
          icon={FileExclamationPointIcon}
          class="w-5 h-5"
          strokeWidth={2}
          size={24}
        />
        Duplicate Download Detected
      </Dialog.Title>
      <Dialog.Description class="pt-2">
        A completed file with the same name and size already exists in your history.
        How would you like to proceed?
      </Dialog.Description>
    </Dialog.Header>

    <div class="grid gap-3 py-4">
      <Button
        variant="outline"
        class="justify-start gap-3 h-auto py-3 border-amber-500/30 hover:bg-amber-500/10 hover:text-amber-600 hover:border-amber-500/50 transition-all"
        onclick={handleOverwrite}
      >
        <div class="p-2 bg-amber-100 dark:bg-amber-900/30 rounded-full">
          <HugeiconsIcon
            icon={FileExclamationPointIcon}
            class="w-4 h-4"
            strokeWidth={2}
            size={24}
          />
        </div>
        <div class="flex flex-col items-start gap-0.5">
          <span class="font-semibold text-foreground">Overwrite / Repair</span>
          <span class="text-xs text-muted-foreground font-normal text-left text-wrap">
            Verifies the existing file and re-downloads only the missing or corrupted parts.
          </span>
        </div>
      </Button>

      <Button
        variant="outline"
        class="justify-start gap-3 h-auto py-3 hover:bg-primary/5 hover:border-primary/40 transition-all"
        onclick={handleDownloadAgain}
      >
        <div class="p-2 bg-primary/10 rounded-full">
          <HugeiconsIcon
            icon={FileExclamationPointIcon}
            class="w-4 h-4"
            strokeWidth={2}
            size={24}
          />
        </div>
        <div class="flex flex-col items-start gap-0.5">
          <span class="font-semibold text-foreground">Download Again</span>
          <span class="text-xs text-muted-foreground font-normal text-left text-wrap">
            Keeps the old file and saves this one as a new copy (e.g. <code>file.1.mp4</code>).
          </span>
        </div>
      </Button>
    </div>

    <Dialog.Footer>
      <Button variant="ghost" onclick={handleCancel}>
        Cancel
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
