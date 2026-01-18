const { test, expect } = require("@playwright/test");

/**
 * BMV Dashboard - API Tests
 * Tests the backend API endpoints through the dashboard
 */

test.describe("BMV Dashboard API Tests", () => {
    const BASE_URL = "http://localhost";
    const PASSWORD = "admin123";
    let authToken = "";

    // Helper: Get auth token via login
    async function getAuthToken(page) {
        await page.goto(`${BASE_URL}/login`);
        await page.fill('input[type="password"]', PASSWORD);
        await page.click('button:has-text("Establish Connection")');
        await page.waitForURL(`${BASE_URL}/`);

        // Get token from localStorage
        authToken = await page.evaluate(() => localStorage.getItem("token"));
        return authToken;
    }

    // ============================================================
    // API ENDPOINT TESTS
    // ============================================================
    test.describe("API Endpoints", () => {
        test("GET /api/stats should return dashboard stats", async ({
            page,
            request,
        }) => {
            const token = await getAuthToken(page);

            const response = await request.get(`${BASE_URL}/api/stats`, {
                headers: {
                    Authorization: `Bearer ${token}`,
                },
            });

            expect(response.status()).toBe(200);
            const data = await response.json();

            // Verify expected fields
            expect(data).toHaveProperty("pivot_price");
            expect(data).toHaveProperty("sol_balance");
            expect(data).toHaveProperty("usdc_balance");
        });

        test("GET /api/latency should return latency data", async ({
            page,
            request,
        }) => {
            const token = await getAuthToken(page);

            const response = await request.get(`${BASE_URL}/api/latency`, {
                headers: {
                    Authorization: `Bearer ${token}`,
                },
            });

            expect(response.status()).toBe(200);
            const data = await response.json();

            // Should be an object with service names as keys
            expect(typeof data).toBe("object");
        });

        test("GET /api/holders should return token holder data", async ({
            page,
            request,
        }) => {
            const token = await getAuthToken(page);

            const response = await request.get(`${BASE_URL}/api/holders`, {
                headers: {
                    Authorization: `Bearer ${token}`,
                },
            });

            expect(response.status()).toBe(200);
            const data = await response.json();

            // Verify expected structure
            expect(data).toHaveProperty("holders");
            expect(data).toHaveProperty("total_supply");
            expect(data).toHaveProperty("top_10_concentration");
            expect(data).toHaveProperty("top_20_concentration");
            expect(data).toHaveProperty("largest_holder_percent");
            expect(Array.isArray(data.holders)).toBeTruthy();
        });

        test("GET /api/wallets should return wallet list", async ({
            page,
            request,
        }) => {
            const token = await getAuthToken(page);

            const response = await request.get(`${BASE_URL}/api/wallets`, {
                headers: {
                    Authorization: `Bearer ${token}`,
                },
            });

            expect(response.status()).toBe(200);
            const data = await response.json();

            // Should be an array of wallets
            expect(Array.isArray(data)).toBeTruthy();
        });

        test("GET /api/orderbook should return order book data", async ({
            page,
            request,
        }) => {
            const token = await getAuthToken(page);

            const response = await request.get(`${BASE_URL}/api/orderbook`, {
                headers: {
                    Authorization: `Bearer ${token}`,
                },
            });

            expect(response.status()).toBe(200);
            const data = await response.json();

            // Should have bids and asks
            expect(data).toHaveProperty("bids");
            expect(data).toHaveProperty("asks");
        });

        test("API should reject requests without auth token", async ({
            request,
        }) => {
            const response = await request.get(`${BASE_URL}/api/stats`);

            // Should return 401 Unauthorized
            expect(response.status()).toBe(401);
        });

        test("API should reject requests with invalid token", async ({
            request,
        }) => {
            const response = await request.get(`${BASE_URL}/api/stats`, {
                headers: {
                    Authorization: "Bearer invalid-token-here",
                },
            });

            // Should return 401 Unauthorized
            expect(response.status()).toBe(401);
        });
    });

    // ============================================================
    // DATA VALIDATION TESTS
    // ============================================================
    test.describe("Data Validation", () => {
        test("Stats API should return valid numeric values", async ({
            page,
            request,
        }) => {
            const token = await getAuthToken(page);

            const response = await request.get(`${BASE_URL}/api/stats`, {
                headers: {
                    Authorization: `Bearer ${token}`,
                },
            });

            const data = await response.json();

            // Validate numeric fields
            expect(typeof data.pivot_price).toBe("number");
            expect(typeof data.sol_balance).toBe("number");
            expect(typeof data.usdc_balance).toBe("number");
            expect(data.sol_balance).toBeGreaterThanOrEqual(0);
            expect(data.usdc_balance).toBeGreaterThanOrEqual(0);
        });

        test("Holders API should return valid concentration percentages", async ({
            page,
            request,
        }) => {
            const token = await getAuthToken(page);

            const response = await request.get(`${BASE_URL}/api/holders`, {
                headers: {
                    Authorization: `Bearer ${token}`,
                },
            });

            const data = await response.json();

            // Concentration should be between 0 and 100
            expect(data.top_10_concentration).toBeGreaterThanOrEqual(0);
            expect(data.top_10_concentration).toBeLessThanOrEqual(100);
            expect(data.top_20_concentration).toBeGreaterThanOrEqual(0);
            expect(data.top_20_concentration).toBeLessThanOrEqual(100);
        });

        test("Latency data should have valid service structure", async ({
            page,
            request,
        }) => {
            const token = await getAuthToken(page);

            const response = await request.get(`${BASE_URL}/api/latency`, {
                headers: {
                    Authorization: `Bearer ${token}`,
                },
            });

            const data = await response.json();

            // Each service should have an array of measurements
            for (const service in data) {
                expect(Array.isArray(data[service])).toBeTruthy();
                if (data[service].length > 0) {
                    const measurement = data[service][0];
                    expect(measurement).toHaveProperty("timestamp");
                    expect(measurement).toHaveProperty("latency_ms");
                    expect(measurement).toHaveProperty("status");
                }
            }
        });
    });

    // ============================================================
    // PERFORMANCE TESTS
    // ============================================================
    test.describe("API Performance", () => {
        test("Stats API should respond within 5 seconds", async ({
            page,
            request,
        }) => {
            const token = await getAuthToken(page);

            const startTime = Date.now();
            const response = await request.get(`${BASE_URL}/api/stats`, {
                headers: {
                    Authorization: `Bearer ${token}`,
                },
            });
            const endTime = Date.now();

            expect(response.status()).toBe(200);
            expect(endTime - startTime).toBeLessThan(5000);
        });

        test("Holders API should respond within 10 seconds", async ({
            page,
            request,
        }) => {
            const token = await getAuthToken(page);

            const startTime = Date.now();
            const response = await request.get(`${BASE_URL}/api/holders`, {
                headers: {
                    Authorization: `Bearer ${token}`,
                },
            });
            const endTime = Date.now();

            expect(response.status()).toBe(200);
            expect(endTime - startTime).toBeLessThan(10000);
        });
    });
});
