<script lang="ts">
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";
  import type { Aria2Options } from "$lib/aria2/types";
	import type { ItemFieldDefinition } from "$lib/components/settings/options-schema";

  let {
    field,
    draft = $bindable()
  }: {
    field: ItemFieldDefinition;
    draft: Partial<Aria2Options>;
  } = $props();

  const handleInput = (e: Event) => {
    const target = e.target as HTMLInputElement;
    draft[field.key] = target.value as any;
  };

  const handleSwitch = (checked: boolean) => {
    draft[field.key] = (checked ? "true" : "false") as any;
  };
</script>

<div class="flex flex-col gap-2 rounded-xl border border-sidebar-border/40 bg-sidebar-accent/20 p-3" id={`field-${field.key}`}>
  <div class="flex items-center justify-between gap-4">
    <div class="flex flex-col gap-1">
      <Label class="text-sm font-semibold">{field.label}</Label>
      <span class="text-xs text-muted-foreground leading-relaxed">{field.description}</span>
      <span class="text-[10px] font-mono text-primary/60 mt-0.5">--{field.key}</span>
    </div>

    {#if field.type === "boolean"}
      <div class="shrink-0">
        <Switch
          checked={draft[field.key] === "true"}
          onCheckedChange={handleSwitch}
        />
      </div>
    {/if}
  </div>

  {#if field.type === "string" || field.type === "number"}
    <div class="mt-2">
      <Input
        type={field.type === "number" ? "number" : "text"}
        value={draft[field.key] || ""}
        oninput={handleInput}
        placeholder="Default"
        class="h-8 text-sm"
      />
    </div>
  {/if}

  {#if field.type === "select" && field.options}
    <div class="mt-2">
      <select
        class="flex h-8 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
        value={draft[field.key] || ""}
        onchange={handleInput}
      >
        <option value="" disabled>Select an option</option>
        {#each field.options as opt (opt)}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </div>
  {/if}
</div>
