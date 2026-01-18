// @ts-check
const { defineConfig, devices } = require("@playwright/test");

/**
 * BMV Dashboard Playwright Configuration
 * @see https://playwright.dev/docs/test-configuration
 */
module.exports = defineConfig({
    testDir: "./",
    /* Run tests in files in parallel */
    fullyParallel: false,
    /* Fail the build on CI if you accidentally left test.only in the source code */
    forbidOnly: !!process.env.CI,
    /* Retry on CI only */
    retries: process.env.CI ? 2 : 0,
    /* Opt out of parallel tests on CI */
    workers: process.env.CI ? 1 : 2,
    /* Reporter to use */
    reporter: [
        ["list"],
        ["html", { outputFolder: "test-results/html-report" }],
        ["json", { outputFile: "test-results/results.json" }],
    ],
    /* Shared settings for all the projects below */
    use: {
        /* Base URL to use in actions like `await page.goto('/')` */
        baseURL: "http://localhost",
        /* Collect trace when retrying the failed test */
        trace: "on-first-retry",
        /* Take screenshot on failure */
        screenshot: "only-on-failure",
        /* Video recording */
        video: "retain-on-failure",
        /* Default timeout for actions */
        actionTimeout: 10000,
    },

    /* Configure projects for major browsers */
    projects: [
        {
            name: "chromium",
            use: { ...devices["Desktop Chrome"] },
        },
        {
            name: "firefox",
            use: { ...devices["Desktop Firefox"] },
        },
        /* Test against mobile viewports */
        {
            name: "Mobile Chrome",
            use: { ...devices["Pixel 5"] },
        },
    ],

    /* Global timeout */
    timeout: 60000,

    /* Global expect timeout */
    expect: {
        timeout: 10000,
    },

    /* Output folder for test artifacts */
    outputDir: "test-results/artifacts",

    /* Folder for screenshots */
    snapshotDir: "screenshots",
});
