<script lang="ts">
	import { mode } from "mode-watcher";
	import { onMount, tick } from "svelte";
	import { fly } from 'svelte/transition';
	import type { ItemMetaData } from "$lib/aria2/types";
	import { Badge } from "$lib/components/ui/badge";
	import { HugeiconsIcon } from "@hugeicons/svelte";
	import { Button } from "$lib/components/ui/button";
	import { getFriendlyError } from "$lib/aria2/error";
	import { buttonVariants } from "$lib/components/ui/button";
	import { Label } from "$lib/components/ui/label/index.js";
	import { Checkbox } from "$lib/components/ui/checkbox/index.js";
	import { formatSpeed, formatBytes } from "$lib/utils";
	import * as Dialog from "$lib/components/ui/dialog";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
	import * as Table from "$lib/components/ui/table";
	import * as Tooltip from "$lib/components/ui/tooltip/index.js";
	import {
		downloadIcons,
		getItemCategory,
		selectedCategory,
		getDownloadIconName
	} from "$lib/categorize";
	import {
		BadgeInfoIcon,
		ArrowUp02Icon,
		ArrowDown02Icon,
		CloudUploadIcon,
		MoreVerticalCircle01Icon
	} from "@hugeicons/core-free-icons";
	import { aria2, aria2Store } from "$lib/aria2/client.svelte";
	import { selectedGids, clearSelection, toggleSelection, selectExclusive } from "$lib/selection";

	/// Some state varibables
	let currentPage = $state(1);
	let totalPages = $state(1);
	let loading = $state(false);
	let hasMore = $state(true);
	let isDarkMode = $state(false);

	/// For deleting
	let deleteDialogOpen = $state(false);
	let deleteWithFiles = $state(false);
	let gidsToDelete: string[] = $state([]);

	/// Infinity scroll
	const AUTO_SCROLL_MARGIN = 64;
	const AUTO_SCROLL_MAX_SPEED = 18;
	let sentinel: HTMLDivElement | null = $state(null);
	let infiniteObserver: IntersectionObserver | null = $state(null);
	let autoScrollFrame: number | null = $state(null);

	/// Drag and select
	const DRAG_THRESHOLD = 4;
	let dragSelecting = $state(false);
	let lastDragSelectionKey = $state("");
	let dragArmed = $state(false);
	let dragStart = $state({ x: 0, y: 0 });
	let dragCurrent = $state({ x: 0, y: 0 });
	let dragStartPage = $state({ x: 0, y: 0 });
	let suppressNextClick = $state(false);
	let tableContainer: HTMLDivElement | null = $state(null);

	/// showing categorized items
	let visibleItems = $derived(
		$selectedCategory === "everything" ? aria2Store.items
			: aria2Store.items.filter((item) => getItemCategory(item) === $selectedCategory)
	);

	/// Selection by clicks and ctrlkey
	function handleRowClick(event: MouseEvent, gid: string) {
		if (suppressNextClick) {
			suppressNextClick = false;
			return;
		}

		if ((event.target as HTMLElement).closest("button, a")) return;

		if (event.ctrlKey || event.metaKey) {
			toggleSelection(gid);
		} else {
			selectExclusive(gid);
		}
	}

	/// Row styles
	function getPercentage(completed: string | undefined, total: string | undefined): string {
		const completedBytes = parseInt(completed || "0");
		const totalBytes = parseInt(total || "0");

		if (isNaN(completedBytes) || isNaN(totalBytes) || totalBytes === 0) {
			return "0%";
		}

		const percentage = (completedBytes / totalBytes) * 100;
		return percentage.toFixed(2) + "%";
	}

	function getProgressPercent(item: ItemMetaData): number {
		const completedBytes = parseInt(item.completedLength || "0");
		const totalBytes = parseInt(item.totalLength || "0");

		if (isNaN(completedBytes) || isNaN(totalBytes) || totalBytes === 0) {
			return item.status === "complete" ? 100 : 0;
		}

		return Math.max(0, Math.min(100, (completedBytes / totalBytes) * 100));
	}

	function getRowStyle(item: ItemMetaData): string {
		const progress = item.status === "complete" ? 100 : getProgressPercent(item);
		return `
      background-image: linear-gradient(90deg, rgba(179, 137, 163, 0.3) 0%,
				rgba(179, 137, 163, 0.3) ${progress}%, transparent ${progress}%, transparent 100%);
      clip-path: inset(0 round 0.0rem);
    `;
	}

	/// Status label animation
	function getStatusClass(status: string) {
		switch (status) {
			case "active":
				return "animate-pulse";
			case "waiting":
				return "animate-pulse";
			default:
				return "";
		}
	}

	/// Time formatting
	function formatDate(dateString: string) {
		const hasTimezone = /(?:Z|[+-]\d{2}:?\d{2})$/i.test(dateString);
		const date = new Date(hasTimezone ? dateString : `${dateString}Z`);

		if (Number.isNaN(date.getTime())) return dateString;

		const diffMs = Date.now() - date.getTime();
		if (diffMs < 0) {
			const userLocales = typeof navigator !== "undefined" ? navigator.languages : ["en"];
			const parts = new Intl.DateTimeFormat(userLocales, {
				hour: "numeric",
				minute: "2-digit",
				hour12: true,
				day: "numeric",
				month: "short",
				year: "numeric"
			}).formatToParts(date);

			const p = parts.reduce(
				(acc, part) => {
					acc[part.type] = part.value;
					return acc;
				},
				{} as Record<string, string>
			);

			return `${p.hour}:${p.minute} ${p.dayPeriod} ${p.day} ${p.month} ${p.year}`;
		}

		const diffMin = Math.floor(diffMs / 60000);
		const diffHour = Math.floor(diffMs / 3600000);
		const diffDay = Math.floor(diffMs / 86400000);

		if (diffMin < 1) return "just now";
		if (diffMin < 60) return `${diffMin} min ago`;
		if (diffHour < 24) return `${diffHour} hour${diffHour === 1 ? "" : "s"} ago`;
		if (diffDay < 7) return `${diffDay} day${diffDay === 1 ? "" : "s"} ago`;
		if (diffDay === 7) return "last week";

		const userLocales = typeof navigator !== "undefined" ? navigator.languages : ["en"];
		const parts = new Intl.DateTimeFormat(userLocales, {
			hour: "numeric",
			minute: "2-digit",
			hour12: true,
			day: "numeric",
			month: "short",
			year: "numeric"
		}).formatToParts(date);

		const p = parts.reduce(
			(acc, part) => {
				acc[part.type] = part.value;
				return acc;
			},
			{} as Record<string, string>
		);

		return `${p.hour}:${p.minute} ${p.dayPeriod} ${p.day} ${p.month} ${p.year}`;
	}

	/// Get categorized icons
	function getIcon(item: ItemMetaData) {
		const name = getDownloadIconName(item);
		return isDarkMode ? downloadIcons[name].dark : downloadIcons[name].light;
	}

	/// Server actions, deletion
	function promptDelete(gids: string[]) {
		if (gids.length === 0) return;
		gidsToDelete = gids;
		deleteWithFiles = false;
		deleteDialogOpen = true;
	}

	function confirmDeletion() {
		aria2.delete(gidsToDelete, deleteWithFiles);
		deleteWithFiles = false;
		deleteDialogOpen = false;
		if (gidsToDelete === $selectedGids) {
			clearSelection();
		}
	}

	function handleStop(item: ItemMetaData) {
		aria2.stop(item.gid);
	}

	async function handleRetry(item: ItemMetaData) {
	}

	async function handleResume(item: ItemMetaData) {
		await aria2.resume(item.gid);
	}

	async function handlePause(item: ItemMetaData) {
		await aria2.pause(item.gid);
	}

	/// Drag selection
	function getDragRect() {
		const currentPage = {
			x: dragCurrent.x + window.scrollX,
			y: dragCurrent.y + window.scrollY
		};

		return {
			left: Math.min(dragStartPage.x, currentPage.x),
			right: Math.max(dragStartPage.x, currentPage.x),
			top: Math.min(dragStartPage.y, currentPage.y),
			bottom: Math.max(dragStartPage.y, currentPage.y)
		};
	}

	function getVisibleRowGids() {
		if (!tableContainer) return [];

		return Array.from(tableContainer.querySelectorAll<HTMLElement>("[data-row-gid]"))
			.map((el) => el.dataset.rowGid)
			.filter((gid): gid is string => Boolean(gid));
	}

	function applySelection(gids: string[]) {
		clearSelection();

		for (const gid of gids) {
			toggleSelection(gid);
		}
	}

	function updateDragSelection() {
		if (!dragSelecting || !tableContainer) return;

		const rect = getDragRect();

		const selected = getVisibleRowGids().filter((gid) => {
			const el = tableContainer?.querySelector<HTMLElement>(`[data-row-gid="${CSS.escape(gid)}"]`);
			if (!el) return false;

			const rowRect = el.getBoundingClientRect();
			const rowPageRect = {
				left: rowRect.left + window.scrollX,
				right: rowRect.right + window.scrollX,
				top: rowRect.top + window.scrollY,
				bottom: rowRect.bottom + window.scrollY
			};

			return !(
				rowPageRect.right < rect.left ||
				rowPageRect.left > rect.right ||
				rowPageRect.bottom < rect.top ||
				rowPageRect.top > rect.bottom
			);
		});

		const selectionKey = selected.join("\u0000");
		if (selectionKey === lastDragSelectionKey) return;

		lastDragSelectionKey = selectionKey;
		applySelection(selected);
	}

	function handleDragMove(event: MouseEvent) {
		if (!dragArmed) return;

		const dx = Math.abs(event.clientX - dragStart.x);
		const dy = Math.abs(event.clientY - dragStart.y);

		if (!dragSelecting) {
			if (dx < DRAG_THRESHOLD && dy < DRAG_THRESHOLD) {
				return;
			}

			dragSelecting = true;
			clearSelection();
			startAutoScroll();
		}

		dragCurrent = { x: event.clientX, y: event.clientY };
		updateDragSelection();
	}

	function handleDragEnd() {
		if (dragSelecting) {
			suppressNextClick = true;
		}

		dragArmed = false;
		dragSelecting = false;
		lastDragSelectionKey = "";
		stopAutoScroll();

		window.removeEventListener("mousemove", handleDragMove, true);
		window.removeEventListener("mouseup", handleDragEnd, true);
		window.removeEventListener("contextmenu", handleDragContextMenu, true);
	}

	function handleDragContextMenu(event: MouseEvent) {
		if (dragSelecting) {
			event.preventDefault();
		}
	}

	function handleSelectionMouseDown(event: MouseEvent) {
		if (event.button !== 0) return;

		const target = event.target as HTMLElement | null;
		if (!target) return;

		if (target.closest('button, a, input, textarea, select, [role="menuitem"]')) {
			return;
		}

		event.preventDefault();
		event.stopPropagation();

		dragArmed = true;
		dragSelecting = false;
		dragStart = { x: event.clientX, y: event.clientY };
		dragCurrent = { x: event.clientX, y: event.clientY };
		dragStartPage = {
			x: event.clientX + window.scrollX,
			y: event.clientY + window.scrollY
		};
		lastDragSelectionKey = "";

		window.addEventListener("mousemove", handleDragMove, true);
		window.addEventListener("mouseup", handleDragEnd, true);
		window.addEventListener("scroll", handleWindowScroll, true);
		window.addEventListener("contextmenu", handleDragContextMenu, true);

		updateDragSelection();
	}

	/// A little refresh fn
	async function refresh(page = 1) {
		if (loading || (!hasMore && page !== 1)) return;

		loading = true;
		try {
			const json = await aria2.loadInitialData(page);
			if (json?.meta) {
				currentPage = json.meta.currentPage;
				totalPages = json.meta.totalPages;
				hasMore = json.meta.hasMore;
			}
		} finally {
			loading = false;
		}
		clearSelection();
		loading = false;
	}

	/// Infinite scroll
	async function loadNextPage() {
		if (loading || !hasMore) return;

		loading = true;
		try {
			const json = await aria2.loadInitialData(currentPage + 1);
			if (json?.meta) {
				currentPage = json.meta.currentPage;
				totalPages = json.meta.totalPages;
				hasMore = json.meta.hasMore ?? currentPage < totalPages;
			} else {
				hasMore = false;
			}
		} finally {
			loading = false;
		}
	}

	function setupInfiniteObserver() {
		if (typeof IntersectionObserver === "undefined" || !sentinel) return;

		infiniteObserver?.disconnect();

		infiniteObserver = new IntersectionObserver(
			(entries) => {
				if (entries[0]?.isIntersecting && hasMore && !loading) {
					void loadNextPage();
				}
			},
			{
				root: null,
				rootMargin: "200px 0px",
				threshold: 0.1
			}
		);

		infiniteObserver.observe(sentinel);
	}

	async function ensureScrollable() {
		await tick();

		while (hasMore && sentinel && sentinel.getBoundingClientRect().top <= window.innerHeight) {
			await loadNextPage();
			await tick();
		}
	}

	function startAutoScroll() {
		if (autoScrollFrame !== null) return;

		const step = () => {
			if (!dragSelecting) {
				stopAutoScroll();
				return;
			}

			let deltaY = 0;
			const topEdge = AUTO_SCROLL_MARGIN;
			const bottomEdge = window.innerHeight - AUTO_SCROLL_MARGIN;

			if (dragCurrent.y < topEdge) {
				const ratio = (topEdge - dragCurrent.y) / AUTO_SCROLL_MARGIN;
				deltaY = -Math.ceil(ratio * AUTO_SCROLL_MAX_SPEED);
			} else if (dragCurrent.y > bottomEdge) {
				const ratio = (dragCurrent.y - bottomEdge) / AUTO_SCROLL_MARGIN;
				deltaY = Math.ceil(ratio * AUTO_SCROLL_MAX_SPEED);
			}

			if (deltaY !== 0) {
				window.scrollBy({ top: deltaY, left: 0, behavior: "auto" });
				updateDragSelection();
			}

			autoScrollFrame = requestAnimationFrame(step);
		};

		autoScrollFrame = requestAnimationFrame(step);
	}

	function stopAutoScroll() {
		if (autoScrollFrame !== null) {
			cancelAnimationFrame(autoScrollFrame);
			autoScrollFrame = null;
		}
	}

	function handleWindowScroll() {
		if (dragSelecting) {
			updateDragSelection();
		}
	}

	$effect(() => {
		if (sentinel) {
			void tick().then(() => setupInfiniteObserver());
		}
		const savedMode = localStorage.getItem("mode-watcher-mode");
		if (mode.current === "dark" || savedMode === "dark") {
			isDarkMode = true;
		} else {
			isDarkMode = false;
		}
	});

	onMount(() => {
		aria2.connectDdlWs();
		aria2.connectGlobalStatWs();
		window.addEventListener("scroll", handleWindowScroll, true);

		void tick().then(() => setupInfiniteObserver());
		void refresh(1).then(() => ensureScrollable());

		return () => {
			window.removeEventListener("scroll", handleWindowScroll, true);
			stopAutoScroll();
			window.removeEventListener("mousemove", handleDragMove, true);
			window.removeEventListener("mouseup", handleDragEnd, true);
			window.removeEventListener("contextmenu", handleDragContextMenu, true);
			infiniteObserver?.disconnect();
			infiniteObserver = null;
		};
	});
</script>

<div class="space-y-4 mt-4">
	<div class="h-10 border-t border-l border-r my-0 mx-6 bg-card md:mx-10 rounded-t-2xl">
		<!-- import GidStatus from "$lib/aria2/types" -->
		<!-- inlcude the all manually -->
		All | "paused" | "active" | "waiting" | "complete" | "removed" | "stopped" | "error";
	</div>
	<div
		role="button"
		tabindex="0"
		bind:this={tableContainer}
		onmousedown={handleSelectionMouseDown}
		class="rounded-b-2xl border bg-card select-none mx-6 md:mx-10"
	>
		<Table.Root>
			<Table.Body>
				{#each visibleItems as item (item.gid)}
					{console.log(`NAME: ${item.name}`)}
					<Table.Row
						data-row-gid={item.gid}
						onclick={(e) => handleRowClick(e, item.gid)}
						data-selected={$selectedGids.includes(item.gid)}
						style={getRowStyle(item)}
						class="cursor-pointer max-h-fit bg-blend-darken transition-colors hover:bg-muted/50 data-[selected=true]:bg-[#DDA0DD]/40 border-xl"
					>
						<div class="w-12 h-16 flex-row flex justify-center items-center">
							<img src={getIcon(item)} alt="svg-icon" aria-hidden="true" class="w-7 shrink-0" />
						</div>
						<Table.Cell class="min-w-[200px]">
							<div class="flex flex-col min-h-10 justify-center ml-[-40px]">
								<span
									class="font-medium float-left max-w-[200px] sm:max-w-[400px]"
									title={item.name || item.gid}
								>
									{(item.name?.length ?? 0) > 70
										? `${item.name?.slice(0, 60)}...${item.name?.slice(-8)}`
										: item.name || "<Untitled>"}
								</span>

								{#if item.status === "error"}
									<div class="flex items-center mt-0.5 text-[12px] text-muted-foreground gap-2">
										<span
											class="flex items-center font-semibold gap-0.5 text-[12px] text-destructive max-w-[300px]"
										>
											<span class="font-mono flex items-center">
												{#if item.completedLength && item.completedLength !== "0"}
													{getPercentage(item.completedLength!, item.totalLength!)} |
												{/if}
												{#if item.completedLength && item.completedLength !== "0"}
													{formatBytes(item.completedLength)}
												{/if}
												{#if item.totalLength && item.totalLength !== "0"}
													{formatBytes(item.totalLength)}
												{/if}
												{#if item.uploadLength && item.uploadLength !== "0"}
													| <HugeiconsIcon
														icon={CloudUploadIcon}
														size={14}
														color="currentColor"
														strokeWidth={1.5}
													/>
													{formatBytes(item.uploadLength)}
												{/if}
											</span>
										</span>
									</div>
								{:else}
									<div
										class="flex items-center text-[12px] font-medium text-muted-foreground gap-2 mt-1"
									>
										{#if (item.completedLength && item.completedLength !== "0") || item.totalLength !== "0"}
											<span class="flex font-mono font-semibold items-center gap-0.5">
												{#if item.completedLength && item.completedLength !== "0"}
													{getPercentage(item.completedLength!, item.totalLength!)} |
												{/if}
												{#if item.completedLength && item.completedLength !== "0"}
													{formatBytes(item.completedLength)}
												{/if}
												{#if item.totalLength && item.totalLength !== "0"}
													{`${item.completedLength && item.completedLength !== "0" ? "/" : ""} ${formatBytes(item.totalLength)}`}
												{/if}
												{#if item.uploadLength && item.uploadLength !== "0"}
													| <HugeiconsIcon
														icon={CloudUploadIcon}
														size={14}
														color="currentColor"
														strokeWidth={1.5}
													/>
													{formatBytes(item.uploadLength)}
												{/if}
												{#if item.status === "complete" && item.completedAt}
													| {formatDate(item.completedAt)}
												{/if}
											</span>
										{/if}
									</div>
								{/if}
							</div>
						</Table.Cell>

						<Table.Cell class="min-w-[280px]!">
							<div class="gap-2 flex flex-col items-center justify-center">
								<Badge
									variant="outline"
									class={`${getStatusClass(item.status)}
									rounded-xl w-24 shadow-sm`}
								>
									{item.status.toUpperCase()}
								</Badge>
								<span
									class="flex flex-row font-mono align-baseline font-semibold text-[12px] items-center gap-0.5 text-muted-foreground"
								>
									{#if item.downloadSpeed && item.downloadSpeed !== "0"}
										{formatSpeed(item.downloadSpeed)}
										<HugeiconsIcon
											icon={ArrowDown02Icon}
											size={14}
											color="currentColor"
											strokeWidth={2}
										/>
									{/if}
									{#if item.uploadSpeed && item.uploadSpeed !== "0"}
										| {formatSpeed(item.uploadSpeed)}
										<HugeiconsIcon
											icon={ArrowUp02Icon}
											size={14}
											color="currentColor"
											strokeWidth={1.5}
										/>
									{/if}
									{#if item.status === "error"}
										<Tooltip.Provider>
											<Tooltip.Root>
												<Tooltip.Trigger
													class="flex text-destructive items-center cursor-pointer justify-center gap-1"
												>
													{getFriendlyError(item.errorCode)}
													<HugeiconsIcon
														icon={BadgeInfoIcon}
														size={12}
														color="currentColor"
														strokeWidth={2}
													/>
												</Tooltip.Trigger>
												<Tooltip.Content>
													<p class="text-destructive">
														{item.errorMessage}
													</p>
												</Tooltip.Content>
											</Tooltip.Root>
										</Tooltip.Provider>
									{/if}
								</span>
							</div>
						</Table.Cell>

						<Table.Cell class="text-right" onclick={(e) => e.stopPropagation()}>
							<DropdownMenu.Root>
								<DropdownMenu.Trigger
									class={buttonVariants({
										variant: "ghost",
										size: "icon",
										className: "cursor-pointer"
									})}
								>
									<HugeiconsIcon
										icon={MoreVerticalCircle01Icon}
										size={24}
										color="currentColor"
										strokeWidth={1.5}
									/>
									<span class="sr-only">Open menu</span>
								</DropdownMenu.Trigger>
								<DropdownMenu.Content align="end" class="p-3 cursor-pointer text-sm">
									{#if ["active"].includes(item.status)}
										<DropdownMenu.Item
											onclick={() => handlePause(item)}
											class="flex hover:rounded-xl items-center px-2 py-1.5 rounded-md
												pointer-events-auto cursor-pointer hover:bg-muted-foreground/10! hover:text-primary!"
										>
											Pause
										</DropdownMenu.Item>
									{/if}
									{#if ["error", "stopped", "removed"].includes(item.status)}
										<DropdownMenu.Item
											onclick={() => handleRetry(item)}
											class="flex hover:rounded-xl items-center px-2 py-1.5 rounded-md
												pointer-events-auto cursor-pointer hover:bg-muted-foreground/10! hover:text-primary!"
										>
											Retry
										</DropdownMenu.Item>
									{/if}
									{#if ["paused"].includes(item.status)}
										<DropdownMenu.Item
											onclick={() => handleResume(item)}
											class="flex hover:rounded-xl items-center px-2 py-1.5 rounded-md
												pointer-events-auto cursor-pointer hover:bg-muted-foreground/10! hover:text-primary!"
										>
											Resume
										</DropdownMenu.Item>
									{/if}
									{#if ["active", "waiting"].includes(item.status)}
										<DropdownMenu.Item
											title="Stop from downloading"
											onclick={() => handleStop(item)}
											class="flex hover:rounded-xl items-center px-2 py-1.5 rounded-md
												pointer-events-auto cursor-pointer hover:bg-muted-foreground/10! hover:text-primary!"
										>
											Stop
										</DropdownMenu.Item>
									{/if}
									{#if ["active", "seeding", "waiting", "paused"].includes(item.status)}
										<DropdownMenu.Item
											title="Stop from downloading"
											onclick={() => handleStop(item)}
											class="flex hover:rounded-xl items-center px-2 py-1.5 rounded-md
												pointer-events-auto cursor-pointer hover:bg-muted-foreground/10! hover:text-primary!"
										>
											Change options
										</DropdownMenu.Item>
									{/if}
									{#if ["error", "stopped"].includes(item.status)}
										<DropdownMenu.Item
											title="Stop from downloading"
											onclick={() => handleStop(item)}
											class="flex hover:rounded-xl items-center px-2 py-1.5 rounded-md
												pointer-events-auto cursor-pointer hover:bg-muted-foreground/10! hover:text-primary!"
										>
											Set options
										</DropdownMenu.Item>
									{/if}
									{#if ["complete"].includes(item.status)}
										<DropdownMenu.Item
											class="flex hover:rounded-xl items-center px-2 py-1.5 rounded-md
												pointer-events-auto cursor-pointer hover:bg-muted-foreground/10! hover:text-primary!"
										>
											Download again
										</DropdownMenu.Item>
									{/if}
									{#if ["seeding"].includes(item.status)}
										<DropdownMenu.Item
											class="flex hover:rounded-xl items-center px-2 py-1.5 rounded-md
												pointer-events-auto cursor-pointer hover:bg-muted-foreground/10! hover:text-primary!"
										>
											Stop seeding
										</DropdownMenu.Item>
									{/if}
									<DropdownMenu.Separator />
									{#if item.status}
										<DropdownMenu.Item
											onclick={() => {}}
											class="flex hover:rounded-xl items-center px-2 py-1.5 rounded-md
												pointer-events-auto cursor-pointer hover:bg-muted-foreground/10! hover:text-primary!"
										>
											Details
										</DropdownMenu.Item>
									{/if}
									{#if ["complete", "seeding"].includes(item.status)}
										<DropdownMenu.Item
											onclick={() => {}}
											class="flex hover:rounded-xl items-center px-2 py-1.5 rounded-md
												pointer-events-auto cursor-pointer hover:bg-muted-foreground/10! hover:text-primary!"
										>
											Open file
										</DropdownMenu.Item>
									{/if}
									{#if item.kind === "uri" || item.sourceUri?.startsWith("magnet")}
										<DropdownMenu.Item
											onclick={() => navigator.clipboard.writeText(item.sourceUri!)}
											class="flex hover:rounded-xl items-center px-2 py-1.5 rounded-md
												pointer-events-auto cursor-pointer hover:bg-muted-foreground/10! hover:text-primary!"
										>
											Copy uri
										</DropdownMenu.Item>
									{/if}
									<DropdownMenu.Item
										onclick={() => navigator.clipboard.writeText(item.files!)}
										class="flex hover:rounded-xl items-center px-2 py-1.5 rounded-md
											pointer-events-auto cursor-pointer hover:bg-muted-foreground/10! hover:text-primary!"
									>
										Copy path
									</DropdownMenu.Item>
									<DropdownMenu.Item
										onclick={() => navigator.clipboard.writeText(item.gid)}
										class="flex hover:rounded-xl items-center px-2 py-1.5 rounded-md
											pointer-events-auto cursor-pointer hover:bg-muted-foreground/10! hover:text-primary!"
									>
										Copy GID
									</DropdownMenu.Item>
									<DropdownMenu.Separator />
									<DropdownMenu.Item
										class="flex text-destructive hover:rounded-xl items-center px-2 py-1.5 rounded-md
											pointer-events-auto cursor-pointer hover:bg-muted-foreground/10! hover:text-destructive/80"
										onclick={() => promptDelete([item.gid])}
									>
										Delete
									</DropdownMenu.Item>
								</DropdownMenu.Content>
							</DropdownMenu.Root>
						</Table.Cell>
					</Table.Row>
				{:else}
					<Table.Row>
						<Table.Cell colspan={5} class="h-140 text-center text-muted-foreground">
							No downloads found {`in category ${$selectedCategory ? $selectedCategory : "."}`}
						</Table.Cell>
					</Table.Row>
				{/each}
			</Table.Body>
		</Table.Root>
		{#if dragSelecting}
			<div
				class="pointer-events-none fixed z-50 rounded-sm border border-primary/70 bg-primary/20"
				style={`left: ${Math.min(dragStart.x, dragCurrent.x)}px;
					top: ${Math.min(dragStart.y, dragCurrent.y)}px;
					width: ${Math.abs(dragCurrent.x - dragStart.x)}px;
					height: ${Math.abs(dragCurrent.y - dragStart.y)}px;`}
			></div>
		{/if}
		<div bind:this={sentinel} class="h-px w-full" aria-hidden="true"></div>
	</div>
</div>

<Dialog.Root bind:open={deleteDialogOpen}>
	<Dialog.Content class="sm:max-w-[500px] sm:min-h-[350px]">
		<Dialog.Header>
			<Dialog.Title>Confirm Delete</Dialog.Title>
			<Dialog.Description>
				Are you sure you want to delete this download?
				<br /> this action cannot be undone.
				<br /><br />
			</Dialog.Description>
		</Dialog.Header>

		<div class="flex flex-row gap-2">
			<div class="w-4 h-6 mt-1 inline">
				<Checkbox class="cursor-pointer" id="delete-with-files" bind:checked={deleteWithFiles} />
			</div>
			<div class="inline">
				<Label for="delete-with-files" class="cursor-pointer">Delete files too!!??</Label>
				<br />
				<span class="text-[14px] font-normal text-muted-foreground"
					>be careful, it'll delete their actual content from server!!</span
				>
			</div>
		</div>

		<Dialog.Footer class="mt-2">
			<Button class="cursor-pointer" variant="outline" onclick={() => (deleteDialogOpen = false)}>
				Cancel
			</Button>
			<Button class="min-w-24 cursor-pointer" variant="destructive" onclick={confirmDeletion}>
				Delete
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
