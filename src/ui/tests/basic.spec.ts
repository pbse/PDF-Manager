import { test, expect } from '@playwright/test';

test.describe('PDF Manager UI', () => {
  test('should load the app and show the welcome screen', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('h2')).toContainText('Select a tool to begin');
  });

  test('should switch to Split tool', async ({ page }) => {
    await page.goto('/');
    await page.click('button[title="Split"]');
    await expect(page.locator('h2')).toContainText('Split');
  });

  test('should switch to Extract tool', async ({ page }) => {
    await page.goto('/');
    await page.click('button[title="Extract & AI"]');
    await expect(page.locator('h2')).toContainText('Extract & AI');
  });
});
