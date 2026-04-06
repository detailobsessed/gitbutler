<script lang="ts">
	import { goto } from "$app/navigation";
	import { workspacePath } from "$lib/routes/routes.svelte";
	import type { PageData } from "./$types";

	const { data }: { data: PageData } = $props();

	$effect(() => {
		goto(workspacePath(data.projectId), { replaceState: true });
	});
</script>
