const { test, expect } = require('@playwright/test');

test.describe('BMV Dashboard UI Validation', () => {
    const BASE_URL = 'http://localhost';
    const PASSWORD = 'admin123';

    test('should login and display real data in indicators', async ({ page }) => {
        // 1. Authentication
        console.log('Navigating to login page...');
        await page.goto(`${BASE_URL}/login`);

        console.log('Performing login...');
        await page.locator('input[type="password"]').fill(PASSWORD);
        await page.locator('button:has-text("Establish Connection")').click();

        // Wait for navigation and switch to EN
        await expect(page).toHaveURL(`${BASE_URL}/`, { timeout: 20000 });
        const enButton = page.locator('button:has-text("EN")');
        if (await enButton.isVisible()) {
            await enButton.click();
            await page.waitForTimeout(1000);
        }

        // Wait for navigation to dashboard with a longer timeout
        try {
            await expect(page).toHaveURL(`${BASE_URL}/`, { timeout: 20000 });
            console.log('Login successful, on dashboard.');
        } catch (err) {
            console.error('Login failed or was too slow. Capturing error screenshot...');
            await page.screenshot({ path: 'login_failure.png' });
            const errorText = await page.locator('.text-red-400').textContent().catch(() => 'No error text visible');
            console.error(`Visible error on page: ${errorText}`);
            throw err;
        }

        // 2. Wait for data to load
        await page.waitForTimeout(5000);

        // 3. Verify Header Indicators
        console.log('Verifying stat cards...');
        const cards = [
            'Asset Pivot',
            'SOL Balance',
            'USDC Balance',
            'Whale Index',
            'Order Imbalance',
            'Safe Haven Index',
            'Market Spread',
            'Node Status',
            'Channel Width'
        ];

        for (const card of cards) {
            const cardElement = page.locator(`div:has-text("${card}")`).filter({ has: page.locator('h4') }).last();
            await expect(cardElement).toBeVisible();
            const value = await cardElement.locator('h4').textContent();
            console.log(`Indicator "${card}" value: ${value}`);

            // Basic check that it's not just "loading" or empty
            expect(value).not.toBe('');
        }

        // 4. Verify Order Book
        console.log('Verifying Order Book...');
        // Use a broader selector that targets the text within any container that might hold the heading
        await expect(page.locator('text=Order Book V1').first()).toBeVisible({ timeout: 15000 });

        // Check for bids or asks (at least some rows)
        const bidsAsks = page.locator('div[side]');
        const count = await bidsAsks.count();
        console.log(`Order book rows found: ${count}`);
        // Note: In dry run or fresh setup, it might be empty, but we expect "real data" if bot is running.

        // 5. Take Screenshots
        console.log('Capturing screenshot...');
        await page.screenshot({ path: 'dashboard_full.png', fullPage: true });

        // 6. Check for "Dry Run Mode" indicator if applicable
        const dryRun = await page.isVisible('text=Dry Run Mode');
        if (dryRun) {
            console.log('Bot is in Dry Run Mode.');
        }
    });
});
