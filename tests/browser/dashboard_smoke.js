const { chromium } = require('playwright');

(async () => {
    const browser = await chromium.launch();
    const page = await browser.newPage();

    const GRAFANA_URL = process.env.GRAFANA_URL || 'http://localhost:3000';

    console.log(`Navigating to Grafana at ${GRAFANA_URL}...`);
    try {
        await page.goto(GRAFANA_URL, { waitUntil: 'networkidle' });

        // Check if we are on the login page or dashboard
        const title = await page.title();
        console.log(`Page Title: ${title}`);

        // Check for BMV Bot Dashboard link or content
        const dashboardExists = await page.isVisible('text=BMV Bot Health');
        if (dashboardExists) {
            console.log('✅ BMV Bot Health dashboard found.');

            // Elaborate: Check for specific metric panels (placeholders for actual IDs)
            const pivotPanel = await page.isVisible('text=Last Pivot Price');
            const solBalancePanel = await page.isVisible('text=Total SOL Balance');

            if (pivotPanel && solBalancePanel) {
                console.log('✅ Specific metrics panels found.');
            } else {
                console.log('⚠️ Some metric panels are missing.');
            }
        } else {
            console.log('⚠️ BMV Bot Health dashboard not immediately visible. Might need login.');
        }

    } catch (error) {
        console.error(`❌ Failed to connect to Grafana: ${error.message}`);
        process.exit(1);
    }

    await browser.close();
    console.log('Smoke test complete.');
})();
