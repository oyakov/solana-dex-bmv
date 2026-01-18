const { test, expect } = require('@playwright/test');

test.describe('BMV Dashboard Russian Support', () => {
    const BASE_URL = 'http://localhost';
    const PASSWORD = 'admin123';

    test('should switch between Russian and English languages', async ({ page }) => {
        // 1. Authentication
        console.log('Navigating to login page...');
        await page.goto(`${BASE_URL}/login`);

        console.log('Performing login...');
        await page.locator('input[type="password"]').fill(PASSWORD);
        await page.locator('button:has-text("Establish Connection")').click();

        // Wait for dashboard to load
        await expect(page).toHaveURL(`${BASE_URL}/`, { timeout: 20000 });
        console.log('Login successful.');

        // 2. Check Default Language (should be RU based on LanguageProvider logic)
        console.log('Verifying default language (RU)...');
        // "Trading Command Center" in RU is "Торговый командный центр"
        await expect(page.locator('h2:has-text("Торговый командный центр")')).toBeVisible();
        await expect(page.locator('button:has-text("Запуск сетки")')).toBeVisible();

        // 3. Switch to English
        console.log('Switching to English...');
        await page.locator('button:has-text("EN")').click();

        // 4. Verify English labels
        console.log('Verifying English labels...');
        await expect(page.locator('h2:has-text("Trading Command Center")')).toBeVisible();
        await expect(page.locator('button:has-text("Deploy Grid")')).toBeVisible();

        // 5. Switch back to Russian
        console.log('Switching back to Russian...');
        await page.locator('button:has-text("RU")').click();

        // 6. Verify Russian labels again
        console.log('Verifying Russian labels again...');
        await expect(page.locator('h2:has-text("Торговый командный центр")')).toBeVisible();

        // 7. Verify Sidebar navigation items in Russian
        await expect(page.locator('text=Командный центр').first()).toBeVisible();
        await expect(page.locator('text=Рой кошельков').first()).toBeVisible();

        console.log('Language switching verified successfully.');
    });
});
