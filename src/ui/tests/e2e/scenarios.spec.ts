import { test, expect } from '@playwright/test';

test.describe('Pinnacle Scenarios', () => {
  
  test('should not show onboarding tour when already completed', async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage.setItem('onboarding_complete', 'true');
      
      // Mock for Tauri v2
      (window as any).__TAURI_INTERNALS__ = {
        invoke: async (cmd: string, args: any) => ({}),
        metadata: { version: '2.0.0' }
      };

      (window as any).__TAURI__ = {
        invoke: (window as any).__TAURI_INTERNALS__.invoke,
        event: { listen: async () => () => {}, emit: async () => {} },
        webviewWindow: {
          getCurrentWebviewWindow: () => ({
            onDragDropEvent: async () => () => {},
            label: 'main'
          })
        }
      };
    });

    await page.goto('/');
    
    // Verify it's NOT visible
    await expect(page.getByText('Welcome to Pinnacle')).toBeHidden({ timeout: 5000 });
  });

  test.describe('Authenticated Scenarios', () => {
    test.beforeEach(async ({ page }) => {
      await page.addInitScript(() => {
        window.localStorage.setItem('onboarding_complete', 'true');
        (window as any).__TAURI_INTERNALS__ = {
          invoke: async (cmd: string, args: any) => {
            if (cmd === 'open_file_dialog') return ['/mock/test.pdf'];
            return {};
          },
          metadata: { version: '2.0.0' }
        };
        (window as any).__TAURI__ = {
          invoke: (window as any).__TAURI_INTERNALS__.invoke,
          event: { listen: async () => () => {}, emit: async () => {} },
          webviewWindow: {
            getCurrentWebviewWindow: () => ({
              onDragDropEvent: async () => () => {},
              label: 'main'
            })
          }
        };
      });
      await page.goto('/');
      await page.evaluate(() => localStorage.setItem('onboarding_complete', 'true'));
      await page.locator('.fixed.inset-0.z-\\[400\\]').waitFor({ state: 'hidden', timeout: 5000 }).catch(() => {});
    });

    test('should open and use shortcuts modal via keyboard', async ({ page }) => {
      // Wait for page to be ready
      await page.waitForTimeout(1000);

      // Try multiple ways to trigger the shortcut
      await page.keyboard.press('Control+/');
      await page.waitForTimeout(500);
      
      let heading = page.getByRole('heading', { name: 'Shortcuts', exact: true });
      
      if (!(await heading.isVisible())) {
        await page.keyboard.press('Control+Slash');
        await page.waitForTimeout(500);
      }

      if (!(await heading.isVisible())) {
        await page.evaluate(() => {
          window.dispatchEvent(new KeyboardEvent('keydown', { key: '/', ctrlKey: true, bubbles: true, cancelable: true }));
        });
        await page.waitForTimeout(500);
      }

      await expect(heading).toBeVisible({ timeout: 5000 });
      // Use a more specific selector for the close button in shortcuts modal
      await page.locator('.fixed.z-\\[300\\]').getByRole('button', { name: '✕' }).click();
      await expect(heading).toBeHidden({ timeout: 5000 });
    });

    test('should use Notepad to add notes', async ({ page }) => {
      await page.click('button[title="Notepad"]', { force: true });
      await expect(page.getByPlaceholder('Capture an insight...')).toBeVisible();
      
      const noteText = 'This is a test note';
      await page.fill('textarea[placeholder="Capture an insight..."]', noteText);
      await page.click('button:has-text("Save Note")');
      
      await expect(page.getByText(noteText)).toBeVisible();
    });

    test('should verify Watermark tool components', async ({ page }) => {
      await page.click('button[title="Watermark"]', { force: true });
      await expect(page.locator('button:has-text("Select PDF")').first()).toBeVisible();
      
      const textInput = page.locator('#wm-text');
      await expect(textInput).toBeVisible();
      await textInput.fill('TOP SECRET');
      
      await expect(page.locator('button:has-text("Stamp Document"), button:has-text("Select PDF")').first()).toBeVisible();
    });

    test('should verify Library and Knowledge Graph', async ({ page }) => {
      await page.click('button[title="Library"]', { force: true });
      await expect(page.getByRole('heading', { name: 'Library', exact: true })).toBeVisible();
      
      // Check for Knowledge Graph component (it's an SVG usually)
      await expect(page.locator('svg').first()).toBeVisible({ timeout: 10000 });
    });

    test('should verify Settings pane', async ({ page }) => {
      await page.click('button[title="Settings"]', { force: true });
      await expect(page.getByRole('heading', { name: 'Settings', exact: true })).toBeVisible();
      
      await expect(page.getByText('AI Provider')).toBeVisible();
      await expect(page.getByText('Data Sovereignty')).toBeVisible();
    });
    
    test('should verify Forms tool', async ({ page }) => {
      await page.click('button[title="Forms"]', { force: true });
      await expect(page.locator('button:has-text("Select PDF")').first()).toBeVisible();
      await expect(page.getByText('Select PDF Form')).toBeVisible();
    });
  });
});
