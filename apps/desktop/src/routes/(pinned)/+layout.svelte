<script lang="ts">
	import ProjectPageLayout from "$components/views/ProjectPageLayout.svelte";
	import type { LayoutData } from "./$types";
	import type { Snippet } from "svelte";

	const { data, children }: { data: LayoutData; children: Snippet } = $props();
</script>

<ProjectPageLayout projectId={data.projectId} projectPinned={data.projectPinned}>
	{@render children()}
</ProjectPageLayout>
