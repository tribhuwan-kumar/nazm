<script lang="ts">
  import { onMount } from "svelte";
	import { mode } from "mode-watcher";
	import { aria2, aria2Store } from "$lib/aria2/client.svelte";
  import { systemState } from '$lib/system';
  import { clearSelection } from "$lib/selection";
  import { HugeiconsIcon } from "@hugeicons/svelte";
	import { logout, authState } from "$lib/auth/auth.svelte";
	import type { Aria2GlobalOptions, ItemMetaData } from "$lib/aria2/types";
	import Label from "$lib/components/ui/label/label.svelte";
	import Slider from "$lib/components/ui/slider/slider.svelte";
	import ThemeToggle from "$lib/components/ThemeToggle.svelte";
	import * as Select from "$lib/components/ui/select";
	import * as Collapsible from "$lib/components/ui/collapsible";
	import * as Sidebar from "$lib/components/ui/sidebar/index.js";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
	import { useSidebar } from "$lib/components/ui/sidebar/index.js";
	import GlobalOptions from "$lib/components/settings/GlobalOptions.svelte";
  import {
		downloadIcons,
		getItemCategory,
		selectedCategory,
		getDownloadIconName
	} from '$lib/categorize';
  import {
    SoftwareIcon,
    Folder01Icon,
    ArrowUp01Icon,
    HelpCircleIcon,
    Settings01Icon,
    ArrowDown01Icon,
    BitcoinCircleIcon
  } from "@hugeicons/core-free-icons";

  let timer: ReturnType<typeof setTimeout> | undefined;
  let isDarkMode = false;
  let showMoreDialog = $state(false);
	let username = $derived(authState.username);
	let role = $derived(authState.role);
  let isAria2Alive = $derived($systemState.status?.aria2Alive);

  const typeItems = [
    { key: "everything", label: "Everything" },
		{ key: "torrent", label: "Torrents" },
    { key: "music", label: "Music" },
    { key: "photos", label: "Photos" },
    { key: "programs", label: "Programs" },
    { key: "videos", label: "Videos" },
    { key: "subtitles", label: "Subtitles" },
    { key: "compressed", label: "Compressed" },
    { key: "others", label: "Others" }
  ] as const;

	/// Get categorized icons
  function getIcon(item: typeof typeItems[number]["key"]) {
		return isDarkMode ? downloadIcons[key].dark : downloadIcons[key].light;
  }

  // Unified reactive state object
  let quickSettings = $state({
    log: "",
		dir: "",
    logLevel: "",
    saveSession: "",
    maxConcurrentDownloads: 5,
    maxOverallDownloadLimit: 0,
    maxOverallUploadLimit: 0
  });

	let itemMetadataStore = $derived(aria2Store.items);
  let categoryCounts = $derived({
    everything: itemMetadataStore.length,
    torrent: itemMetadataStore.filter((item) => getItemCategory(item) === "torrent").length,
    music: itemMetadataStore.filter((item) => getItemCategory(item) === "music").length,
    photos: itemMetadataStore.filter((item) => getItemCategory(item) === "photos").length,
    programs: itemMetadataStore.filter((item) => getItemCategory(item) === "programs").length,
    videos: itemMetadataStore.filter((item) => getItemCategory(item) === "videos").length,
    subtitles: itemMetadataStore.filter((item) => getItemCategory(item) === "subtitles").length,
    compressed: itemMetadataStore.filter((item) => getItemCategory(item) === "compressed").length,
    others: itemMetadataStore.filter((item) => getItemCategory(item) === "others").length
  });

	const sidebarState = useSidebar();
  const encodeSpeed = (val: number): string => (val === 0 ? "0" : `${val}K`);

  const decodeSpeed = (val: string | undefined): number => {
    if (!val || val === "0") return 0;
    let multiplier = 1;
    const upper = val.toUpperCase();
    if (upper.endsWith("M")) multiplier = 1024;
    else if (!upper.endsWith("K")) multiplier = 1 / 1024;
    const parsed = Number(val.replace(/[^0-9.]/g, ""));
    return Number.isFinite(parsed) ? Math.round(parsed * multiplier) : 0;
  };

  async function loadGlobalOptions() {
    try {
      const options = await aria2.getGlobalOption();
      console.log("Global options:", options)
      quickSettings.log = options["log"] ?? "";
      quickSettings.dir = options["dir"] ?? "";
      quickSettings.logLevel = options["log-level"] ?? "notice";
      quickSettings.saveSession = options["save-session"] ?? "";
      quickSettings.maxConcurrentDownloads = Number(options["max-concurrent-downloads"] ?? 5);

      // Decode raw aria strings ("5M", "100K", "0") into KiB/s
      quickSettings.maxOverallDownloadLimit = decodeSpeed(options["max-overall-download-limit"]);
      quickSettings.maxOverallUploadLimit = decodeSpeed(options["max-overall-upload-limit"]);
    } catch (error) {
      console.error("Failed to load global options:", error);
    }
  }

  function selectCategory(category: typeof typeItems[number]["key"]) {
    selectedCategory.set(category);
    clearSelection();
  }

  const formatSpeed = (val: number): string => {
    if (val === 0) return "Unlimited";
    if (val < 1024) return `${val} KiB/s`;
    return `${(val / 1024).toFixed(2).replace(/\.00$/, "")} MiB/s`;
  };

  let downloadLimitLabel = $derived(formatSpeed(quickSettings.maxOverallDownloadLimit));
  let uploadLimitLabel = $derived(formatSpeed(quickSettings.maxOverallUploadLimit));

  async function pushSetting(key: keyof Aria2GlobalOptions, value: string) {
    try {
      await aria2.changeGlobalOption({ [key]: value });
    } catch (error) {
      console.error(`Failed to update option ${key}:`, error);
      await loadGlobalOptions();
    }
  }

  onMount(async () => {
    const update = () => {
			const savedMode = localStorage.getItem('mode-watcher-mode')
			if (mode.current === "dark" || savedMode === "dark") {
				isDarkMode = true;
			}
    };
    update();
    loadGlobalOptions();
  });
</script>

<Sidebar.Root>
  <Sidebar.Header class="flex flex-row items-baseline text-[20px]">
    NAZM
    <span class="inline text-[14px] font-medium">
      Yet another Aria2 web ui
    </span>
  </Sidebar.Header>

  <Sidebar.Content>
    <div class="px-2 py-3 space-y-4">
      <Sidebar.Menu>
        <Collapsible.Root open class="group/collapsible">
          <Sidebar.MenuItem>
            <Collapsible.Trigger>
              {#snippet child({ props })}
                <Sidebar.MenuButton {...props} class="w-full cursor-pointer justify-between">
                  <div class="flex items-center gap-2">
                    <HugeiconsIcon icon={Folder01Icon} strokeWidth={2} class="size-4" />
                    <span class="font-medium">Category</span>
                  </div>
                  <HugeiconsIcon
                    icon={ArrowDown01Icon}
                    strokeWidth={2}
                    class="size-4 shrink-0 transition-transform duration-200 group-data-[state=open]/collapsible:rotate-180"
                  />
                </Sidebar.MenuButton>
              {/snippet}
            </Collapsible.Trigger>
            <Collapsible.Content>
              <Sidebar.MenuSub>
                {#each typeItems as item (item.key)}
                  <Sidebar.MenuSubItem>
                    <Sidebar.MenuSubButton
                      onclick={() => selectCategory(item.key)}
                      class={$selectedCategory === item.key ? "bg-sidebar-accent text-sidebar-accent-foreground cursor-pointer" : "cursor-pointer text-muted-foreground"}
                    >
                      <span class={$selectedCategory === item.key ? "text-primary font-medium" : "text-muted-foreground"}>
                        {item.label}
                      </span>
                      <span class={$selectedCategory === item.key ? "text-primary ml-auto text-xs font-semibold" : "ml-auto text-xs text-muted-foreground opacity-70"}>
                        {categoryCounts[item.key] ?? 0}
                      </span>
                    </Sidebar.MenuSubButton>
                  </Sidebar.MenuSubItem>
                {/each}
              </Sidebar.MenuSub>
            </Collapsible.Content>
          </Sidebar.MenuItem>
        </Collapsible.Root>

        <Collapsible.Root open class="group/collapsible">
          <Sidebar.MenuItem>
            <Collapsible.Trigger>
              {#snippet child({ props })}
                <Sidebar.MenuButton {...props} class="w-full cursor-pointer justify-between mt-2">
                  <div class="flex items-center gap-2">
                    <HugeiconsIcon icon={Settings01Icon} strokeWidth={2} class="size-4" />
                    <span class="font-medium">Global settings</span>
                  </div>
                  <HugeiconsIcon
                    icon={ArrowDown01Icon}
                    strokeWidth={2}
                    class="size-4 shrink-0 transition-transform duration-200 group-data-[state=open]/collapsible:rotate-180"
                  />
                </Sidebar.MenuButton>
              {/snippet}
            </Collapsible.Trigger>

            <Collapsible.Content>
              <Sidebar.MenuSub>
                <div class="space-y-3 pt-1">
                  <div class={`rounded-xl border border-sidebar-border bg-sidebar-background/80 p-3 space-y-2
									${!isAria2Alive ? "cursor-not-allowed!" : ""}`}>
                    <div class="flex items-center justify-between">
                      <Label class="block text-xs font-medium tracking-wide text-sidebar-foreground/70">
                        max-concurrent-downloads
                      </Label>
                      <span class="text-xs font-mono text-muted-foreground">{quickSettings.maxConcurrentDownloads}</span>
                    </div>
                    <!-- {should've wrote a debounce function} -->
                    <input
                      type="range"
											disabled={!isAria2Alive}
                      class={`cursor-pointer custom-slider ${!isAria2Alive ? "cursor-not-allowed!" : ""}`}
                      min={1}
                      max={1000}
                      step={1}
                      value={quickSettings.maxConcurrentDownloads}
											oninput={(e) => {
                        const val = Number(e.currentTarget.value);
                        quickSettings.maxConcurrentDownloads = val;
                        if (timer) {
                          clearTimeout(timer);
                        }
                        timer = setTimeout(() => {
                          pushSetting("max-concurrent-downloads", String(val));
                        }, 800);
                      }}
                    />
                  </div>

                  <div class="rounded-xl border border-sidebar-border bg-sidebar-background/80 p-3 space-y-2">
                    <div class="flex items-center justify-between">
                      <Label class="block text-xs font-medium tracking-wide text-sidebar-foreground/70">
                        max-overall-download-limit
                      </Label>
                      <span class="text-xs font-mono text-muted-foreground">{downloadLimitLabel}</span>
                    </div>
                    <input
                      type="range"
                      min={0}
                      max={16384}
                      step={8}
                      value={quickSettings.maxOverallDownloadLimit}
                      oninput={(e) => {
                        const val = Number(e.currentTarget.value);
                        quickSettings.maxOverallDownloadLimit = val;
                        if (timer) {
                          clearTimeout(timer);
                        }
                        timer = setTimeout(() => {
                          pushSetting("max-overall-download-limit", encodeSpeed(val));
                        }, 800);
                      }}
											disabled={!isAria2Alive}
                      class={`cursor-pointer custom-slider ${!isAria2Alive ? "cursor-not-allowed!" : ""}`}
                    />
                  </div>

                  <div class="rounded-xl border border-sidebar-border bg-sidebar-background/80 p-3 space-y-2">
                    <div class="flex items-center justify-between">
                      <Label class="block text-xs font-medium tracking-wide text-sidebar-foreground/70">
                        max-overall-upload-limit
                      </Label>
                      <span class="text-xs font-mono text-muted-foreground">{uploadLimitLabel}</span>
                    </div>
                    <input
                      type="range"
                      min={0}
                      max={16384}
                      step={8}
                      value={quickSettings.maxOverallUploadLimit}
                      oninput={(e) => {
                        const val = Number(e.currentTarget.value);
                        quickSettings.maxOverallUploadLimit = val;
                        if (timer) {
                          clearTimeout(timer);
                        }
                        timer = setTimeout(() => {
                          pushSetting("max-overall-upload-limit", encodeSpeed(val));
                        }, 800);
                      }}
											disabled={!isAria2Alive}
                      class={`cursor-pointer custom-slider ${!isAria2Alive ? "cursor-not-allowed!" : ""}`}
                    />
                  </div>

                  <div class="rounded-xl border border-sidebar-border bg-sidebar-background/80 p-3 space-y-2">
                    <Label class="block text-xs font-medium tracking-wide text-sidebar-foreground/70">
											Dir
                    </Label>
                    <input
                      type="text"
                      value={quickSettings.dir}
                      onchange={(e) => {
                        quickSettings.dir = e.currentTarget.value;
                        pushSetting("dir", e.currentTarget.value);
                      }}
                      class={`w-full rounded-md border border-sidebar-border bg-sidebar-background/80 px-3 py-1.5 text-sm
												outline-none placeholder:text-muted-foreground/50 text-foreground transition-shadow
												focus-visible:ring-1 focus-visible:ring-sidebar-border ${!isAria2Alive ? "cursor-not-allowed!" : ""}`}
											disabled={!isAria2Alive}
                    />
                  </div>

                  <div class="rounded-xl border border-sidebar-border bg-sidebar-background/80 p-3 space-y-2">
                    <Label class="block text-xs font-medium tracking-wide text-sidebar-foreground/70">
                      Log
                    </Label>
                    <input
                      type="text"
                      value={quickSettings.log}
                      onchange={(e) => {
                        quickSettings.log = e.currentTarget.value;
                        pushSetting("log", e.currentTarget.value);
                      }}
                      class={`w-full rounded-md border border-sidebar-border bg-sidebar-background/80 px-3 py-1.5 text-sm outline-none placeholder:text-muted-foreground/50
												text-foreground transition-shadow focus-visible:ring-1 focus-visible:ring-sidebar-border ${!isAria2Alive ? "cursor-not-allowed!" : ""}`}
											disabled={!isAria2Alive}
                    />
                  </div>

                  <div class="rounded-xl border border-sidebar-border bg-sidebar-background/80 p-3 space-y-2">
                    <Label class="block text-xs font-medium tracking-wide text-sidebar-foreground/70">
                      Log level
                    </Label>
                    <Select.Root
                      type="single"
                      value={quickSettings.logLevel}
											disabled={!isAria2Alive}
                      onValueChange={(value: string) => {
                        if (value) {
                          quickSettings.logLevel = value;
                          pushSetting("log-level", value);
                        }
                      }}
                    >
                      <Select.Trigger disabled={!isAria2Alive}
												class="w-full rounded-md border cursor-pointer border-sidebar-border bg-sidebar-background/80 px-3 py-1.5 text-sm text-left flex justify-between
													items-center transition-colors hover:bg-muted/30"
												>
                        {quickSettings.logLevel || "Select log level"}
                      </Select.Trigger>
                      <Select.Content class="bg-popover/50 shadow-md p-1 min-w-[8rem]">
                        <Select.Group>
                          <Select.Item value="debug" class="px-2 py-1 text-sm text-muted-foreground! hover:text-primary! cursor-pointer hover:bg-accent">debug</Select.Item>
                          <Select.Item value="info" class="px-2 py-1 text-sm cursor-pointer hover:bg-accent text-muted-foreground! hover:text-primary!">info</Select.Item>
                          <Select.Item value="notice" class="px-2 py-1 text-sm cursor-pointer hover:bg-accent text-muted-foreground! hover:text-primary!">notice</Select.Item>
                          <Select.Item value="warn" class="px-2 py-1 text-sm cursor-pointer hover:bg-accent text-muted-foreground! hover:text-primary!">warn</Select.Item>
                          <Select.Item value="error" class="px-2 py-1 text-sm cursor-pointer hover:bg-accent text-muted-foreground! hover:text-primary!">error</Select.Item>
                        </Select.Group>
                      </Select.Content>
                    </Select.Root>
                  </div>

                  <div class="rounded-xl border border-sidebar-border bg-sidebar-background/80 p-3 space-y-2">
                    <Label class="block text-xs font-medium tracking-wide text-sidebar-foreground/70">
                      Save session
                    </Label>
                    <input
											disabled={!isAria2Alive}
                      type="text"
                      value={quickSettings.saveSession}
                      onchange={(e) => {
                        quickSettings.saveSession = e.currentTarget.value;
                        pushSetting("save-session", e.currentTarget.value);
                      }}
                      class={`w-full rounded-md border border-sidebar-border bg-sidebar-background/80 px-3 py-1.5 text-sm outline-none placeholder:text-muted-foreground/50
												text-foreground transition-shadow focus-visible:ring-1 focus-visible:ring-sidebar-border ${!isAria2Alive ? "cursor-not-allowed!" : ""}`}
                    />
                  </div>

                  <button
                    type="button"
                    onclick={() => {
											showMoreDialog = true
											// sidebarState.toggle()
										}}
										disabled={!isAria2Alive}
                    class={`w-full rounded-xl border border-sidebar-border bg-sidebar-background/80 px-3 py-3
										text-left text-sm font-semibold transition-colors  hover:bg-sidebar-accent hover:text-sidebar-accent-foreground
										${!isAria2Alive ? "cursor-not-allowed!" : ""} cursor-pointer group`}
                  >
                    <span
                      class="group-hover:text-primary transition-colors"
                    >
                      Global settings
                    </span>
                    <span class="block text-xs font-normal text-muted-foreground mt-0.5">
                      Set additional global configuration parameters.
                    </span>
                  </button>
                </div>
              </Sidebar.MenuSub>
            </Collapsible.Content>
          </Sidebar.MenuItem>
        </Collapsible.Root>

        <Collapsible.Root open class="group/collapsible">
          <Sidebar.MenuItem>
            <Collapsible.Trigger>
              {#snippet child({ props })}
                <Sidebar.MenuButton {...props} class="w-full cursor-pointer justify-between mt-2">
                  <div class="flex items-center gap-2">
                    <HugeiconsIcon icon={SoftwareIcon} strokeWidth={2} class="size-4" />
                    <span class="font-medium">App info</span>
                  </div>
                  <HugeiconsIcon
                    icon={ArrowDown01Icon}
                    strokeWidth={2}
                    class="size-4 shrink-0 transition-transform duration-200 group-data-[state=open]/collapsible:rotate-180"
                  />
                </Sidebar.MenuButton>
              {/snippet}
            </Collapsible.Trigger>
            <Collapsible.Content>
              <Sidebar.MenuSub>
                <Sidebar.MenuSubItem>
                  <Sidebar.MenuSubButton class="cursor-pointer text-muted-foreground!">
                    Aria2
                  </Sidebar.MenuSubButton>
                </Sidebar.MenuSubItem>
                <Sidebar.MenuSubItem>
                  <Sidebar.MenuSubButton class="cursor-pointer text-muted-foreground!">
                    NAZM
                  </Sidebar.MenuSubButton>
                </Sidebar.MenuSubItem>
                <Sidebar.MenuSubItem>
                  <Sidebar.MenuSubButton class="cursor-pointer text-muted-foreground!">
                    Storage
                  </Sidebar.MenuSubButton>
                </Sidebar.MenuSubItem>
              </Sidebar.MenuSub>
            </Collapsible.Content>
          </Sidebar.MenuItem>
        </Collapsible.Root>

        <div class="my-2 border-t border-sidebar-border"></div>

        <Sidebar.MenuItem>
          <Sidebar.MenuButton class="w-full justify-start cursor-pointer">
            <HugeiconsIcon icon={HelpCircleIcon} strokeWidth={2} class="size-4 mr-2" />
            <span>Help</span>
          </Sidebar.MenuButton>
        </Sidebar.MenuItem>

        <Sidebar.MenuItem>
          <Sidebar.MenuButton class="w-full justify-start cursor-pointer">
            <HugeiconsIcon icon={BitcoinCircleIcon} strokeWidth={2} class="size-4 mr-2" />
            <span>Donations</span>
          </Sidebar.MenuButton>
        </Sidebar.MenuItem>

        <div class="my-2 border-t border-sidebar-border"></div>

        <Sidebar.MenuItem class="pointer-events-none!">
          <Sidebar.MenuButton class="min-h-12 pointer-events-none! flex justify-center items-center cursor-pointer hover:bg-transparent!">
            <Sidebar.MenuSubItem class="pointer-events-none!">
              <ThemeToggle class="cursor-pointer pointer-events-auto"/>
            </Sidebar.MenuSubItem>
          </Sidebar.MenuButton>
        </Sidebar.MenuItem>


      </Sidebar.Menu>
    </div>
  </Sidebar.Content>

  <Sidebar.Footer>
    <Sidebar.Menu>
      <Sidebar.MenuItem>
        <DropdownMenu.Root>
          <DropdownMenu.Trigger>
            {#snippet child({ props })}
              <Sidebar.MenuButton
                {...props}
                class="w-full rounded-xl h-16! relative cursor-pointer border border-sidebar-border/60 bg-sidebar-background/80 px-3
									py-6 shadow-sm transition-colors data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
								title={`'${username}' has '${role}' role`}
              >
                <div class="flex min-w-0 flex-1 items-center gap-3">
                  <div class="flex h-9 w-9 items-center justify-center rounded-full bg-[#DDA0DD] text-xs font-semibold text-sidebar-primary-foreground">
										{username?.charAt(0).toUpperCase()}
                  </div>
                  <div class="min-w-0 text-left">
                    <p class="text-[16px] font-medium text-primary">{username}</p>
										<span class="absolute top-9 right-[60px]">
											<span class="text-[12px] text-sidebar-foreground/90 font-light">as</span>
											<span class="text-sidebar-foreground/60 text-[14px]">{role}</span>
										</span>
                    <!-- <p class="truncate text-xs text-sidebar-foreground/60">{role}</p> -->
                  </div>
                </div>
                <HugeiconsIcon
									icon={ArrowUp01Icon}
									strokeWidth={2}
									class="ml-2 size-4 shrink-0"
								/>
              </Sidebar.MenuButton>
            {/snippet}
          </DropdownMenu.Trigger>

          <DropdownMenu.Content side="top" align="center" class="w-(--bits-dropdown-menu-anchor-width) bg-popover/50 p-2 pointer-events-none!">
            <DropdownMenu.Group>
              <DropdownMenu.Label class="px-2 py-1.5 mt-1 text-xs font-semibold text-muted-foreground pointer-events-auto">
                User Info
              </DropdownMenu.Label>
              <DropdownMenu.Item class="flex hover:rounded-xl items-center px-2 py-1.5 text-sm rounded-md
								pointer-events-auto cursor-pointer hover:bg-accent/10! text-muted-foreground! hover:text-primary!">
                <span>
                  Account
									<!-- change password-->
									<!-- account details, account created and modified at-->
									<!-- total number of completed, uncompleted download files-->
									<!-- total download sucessfully download size-->
									<!-- total number of video, audio, image, torrents files downloaded along with sizes-->
									<!-- -->
									<!-- total size of seeded means `upload_length`-->
                </span>
              </DropdownMenu.Item>
              <DropdownMenu.Item class="flex items-center hover:rounded-xl px-2 py-1.5 text-sm rounded-md
								pointer-events-auto cursor-pointer hover:bg-accent/10! text-muted-foreground! hover:text-primary!">
                <span>
                  Create user
                </span>
              </DropdownMenu.Item>
              <DropdownMenu.Item class="flex items-center hover:rounded-xl px-2 py-1.5 text-sm rounded-md
								pointer-events-auto cursor-pointer hover:bg-accent/10! text-muted-foreground! hover:text-primary!">
                <span>
									Settings
                </span>
              </DropdownMenu.Item>
            </DropdownMenu.Group>
            <DropdownMenu.Separator class="h-px bg-border my-1" />
            <DropdownMenu.Item
							onclick={logout}
							class="flex items-center hover:rounded-xl hover:bg-accent/10! px-2 py-1.5 pointer-events-auto text-sm rounded-md cursor-pointer
							text-destructive focus:bg-destructive/10 focus:text-destructive"
						>
              <span>Sign out</span>
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Root>
      </Sidebar.MenuItem>
    </Sidebar.Menu>
  </Sidebar.Footer>
</Sidebar.Root>

<Sidebar.Inset>
  <div class="min-h-screen bg-gradient-to-br from-background via-background to-muted/20">
    <slot />
  </div>
</Sidebar.Inset>

<GlobalOptions bind:open={showMoreDialog} onsave={loadGlobalOptions} />




