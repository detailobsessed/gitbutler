import { getBaseURL, type GitButler, startGitButler } from "../src/setup.ts";
import { test } from "../src/test.ts";
import { dragAndDropByLocator, waitForTestId } from "../src/util.ts";
import { expect, type Page } from "@playwright/test";

let gitbutler: GitButler;

test.use({
	baseURL: getBaseURL(),
});

test.afterEach(async () => {
	await gitbutler?.destroy();
});

/**
 * Set up a workspace with branch1 (4 commits) and branch2 (2 commits) applied.
 * branch1 modifies a_file, branch2 modifies b_file — they are independent.
 */
async function setupWorkspaceWithTwoBranches(page: Page, context: any, testInfo: any) {
	const workdir = testInfo.outputPath("workdir");
	const configdir = testInfo.outputPath("config");
	gitbutler = await startGitButler(workdir, configdir, context);

	await gitbutler.runScript("project-with-stacks.sh");
	await gitbutler.runScript("apply-upstream-branch.sh", ["branch1", "local-clone"]);
	await gitbutler.runScript("apply-upstream-branch.sh", ["branch2", "local-clone"]);

	await page.goto("/");
	await waitForTestId(page, "workspace-view");

	// Verify we have two stacks
	const stacks = page.getByTestId("stack");
	await expect(stacks).toHaveCount(2);
}

/**
 * Get the modifier key for multi-select (Meta on macOS, Control elsewhere).
 */
function getModifierKey(): "Meta" | "Control" {
	return process.platform === "darwin" ? "Meta" : "Control";
}

test("should move multiple selected commits to a different branch via drag and drop", async ({
	page,
	context,
}, testInfo) => {
	test.setTimeout(120_000);
	await setupWorkspaceWithTwoBranches(page, context, testInfo);

	// Identify the two stacks by their branch header (not hasText, since commit
	// messages contain "branch2" and would match both stacks after the move).
	const stack1 = page
		.getByTestId("stack")
		.filter({ has: page.getByTestId("branch-header").filter({ hasText: "branch1" }) });
	const stack2 = page
		.getByTestId("stack")
		.filter({ has: page.getByTestId("branch-header").filter({ hasText: "branch2" }) });
	await expect(stack1).toBeVisible();
	await expect(stack2).toBeVisible();

	// branch1 should have 4 commits, branch2 should have 2
	const branch1Commits = stack1.getByTestId("commit-row");
	await expect(branch1Commits).toHaveCount(4);
	const branch2Commits = stack2.getByTestId("commit-row");
	await expect(branch2Commits).toHaveCount(2);

	// Move branch2 commits to branch1 (branch2 modifies b_file, so no conflict).
	// Select both branch2 commits.
	const b2Second = branch2Commits.filter({ hasText: "branch2: second commit" });
	const b2First = branch2Commits.filter({ hasText: "branch2: first commit" });

	await b2Second.click();
	const modKey = getModifierKey();
	await b2First.click({ modifiers: [modKey] });

	// Both should be selected
	await expect(b2Second).toHaveClass(/\bselected\b/);
	await expect(b2First).toHaveClass(/\bselected\b/);

	// Drag the selected commits onto branch1's branch header
	const branch1Header = stack1.getByTestId("branch-header").first();
	await dragAndDropByLocator(page, b2Second, branch1Header, { force: true });

	// After the move, branch1 should have 6 commits (original 4 + moved 2)
	await expect(stack1.getByTestId("commit-row")).toHaveCount(6, { timeout: 15_000 });

	// Verify the moved commits appear in branch1
	const branch1CommitsAfter = stack1.getByTestId("commit-row");
	await expect(branch1CommitsAfter.filter({ hasText: "branch2: first commit" })).toHaveCount(1);
	await expect(branch1CommitsAfter.filter({ hasText: "branch2: second commit" })).toHaveCount(1);
});
