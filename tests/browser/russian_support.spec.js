const { test, expect } = require('@playwright/test');

test.describe('BMV Dashboard Russian Support & Chart Controls', () => {
    const BASE_URL = 'http://localhost';
    const PASSWORD = 'admin123';

    test('should switch between Russian and English languages', async ({ page }) => {
        // 1. Authentication
        console.log('Navigating to login page...');
        await page.goto(`${BASE_URL}/login`);
        await page.screenshot({ path: 'test-results/login_page.png' });

        console.log('Performing login...');
        await page.locator('input[type="password"]').fill(PASSWORD);
        await page.locator('button:has-text("Establish Connection")').click();

        // Wait for dashboard to load
        console.log('Waiting for URL redirect...');
        await expect(page).toHaveURL(`${BASE_URL}/`, { timeout: 30000 });
        console.log('Login successful.');
        await page.screenshot({ path: 'test-results/dashboard_loaded.png' });

        // 2. Check Default Language (should be RU)
        console.log('Verifying default language (RU)...');
        const ruTitle = page.locator('h2:has-text("Торговый командный центр")');
        await expect(ruTitle).toBeVisible({ timeout: 15000 });
        console.log('RU language verified.');

        // 3. Switch to English
        console.log('Switching to English...');
        await page.locator('button:has-text("EN")').click();
        await page.waitForTimeout(1000);

        // 4. Verify English labels
        console.log('Verifying English labels...');
        const enTitle = page.locator('h2:has-text("Trading Command Center")');
        await expect(enTitle).toBeVisible({ timeout: 15000 });
        console.log('EN language verified.');

        // 5. Verify Chart Time Scale Selectors
        console.log('Verifying Chart Time Scale Selectors...');
        // Default should be 1D
        const btn1D = page.locator('button:has-text("1D")');
        await expect(btn1D).toHaveClass(/bg-cyan-500/);

        console.log('Clicking 1H...');
        const btn1H = page.locator('button:has-text("1H")');
        await btn1H.click();
        await expect(btn1H).toHaveClass(/bg-cyan-500/);
        await expect(btn1D).not.toHaveClass(/bg-cyan-500/);

        console.log('Clicking 1W...');
        const btn1W = page.locator('button:has-text("1W")');
        await btn1W.click();
        await expect(btn1W).toHaveClass(/bg-cyan-500/);

        console.log('Chart time scale selectors verified.');

        await page.screenshot({ path: 'test-results/final_verified.png' });
        console.log('Tests completed successfully.');
    });
});
