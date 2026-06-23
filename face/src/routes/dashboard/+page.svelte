<script lang="ts">
import { fly } from 'svelte/transition';
import Sidebar from "$lib/components/Sidebar.svelte";
import DlTable from '$lib/components/DlTable.svelte';
import Navbar from '$lib/components/Navbar.svelte';
import ChangeOption from '@/components/settings/ChangeOption.svelte';
import * as SidebarUI from "$lib/components/ui/sidebar/index.js";
</script>

<div class="flex flex-row w-full h-full min-h-svh"
	in:fly={{ y: 100, duration: 800 }}
>
	<SidebarUI.Provider class="w-full h-full min-h-svh">
		<Sidebar />
		<div class="flex min-h-svh w-full flex-col">
			<Navbar />
			<DlTable />
			<!-- <ChangeOption open={true} gid="b266072357bc68ab" kind="torrent" onsave={() => {}}  /> -->
		</div>
	</SidebarUI.Provider>
</div>
