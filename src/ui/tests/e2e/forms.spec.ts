import { test, expect } from '@playwright/test';

test.describe('Pinnacle Form Builder', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage.setItem('onboarding_complete', 'true');
      (window as any).__TAURI_INTERNALS__ = {
        invoke: async (cmd: string, args: any) => {
          console.log(`Mocked invoke: ${cmd}`, args);
          if (cmd === 'open_file_dialog') return ['/mock/test.pdf'];
          if (cmd === 'save_file_dialog') return '/mock/output.pdf';
          if (cmd === 'read_file_bytes') return new Uint8Array([37, 80, 68, 70, 45]);
          if (cmd === 'get_form_fields') return [];
          if (cmd === 'create_form_fields') return {};
          if (cmd === 'shell_open') return {};
          if (cmd === 'pdf_to_text_string') return "Name: ____\nAddress: ____";
          return {};
        },
        metadata: { version: '2.0.0' }
      };
      (window as any).__TAURI__ = {
        invoke: (window as any).__TAURI_INTERNALS__.invoke,
        event: { 
          listen: async () => () => {},
          emit: async () => {}
        },
        webviewWindow: {
          getCurrentWebviewWindow: () => ({
            onDragDropEvent: async () => () => {},
            label: 'main'
          })
        }
      };
    });
    await page.goto('/');
  });

  test('should allow creating a new form field in builder mode', async ({ page }) => {
    // Switch to Forms tool
    await page.click('button[title="Forms"]', { force: true });
    await expect(page.getByTestId('active-pane-forms')).toBeVisible();

    // Toggle Builder Mode
    await page.click('button:has-text("Builder Mode")');
    await expect(page.getByText('Add New Field')).toBeVisible();

    // Select PDF
    await page.click('button:has-text("Select PDF Form")');

    // Mock a selection
    await page.evaluate(() => {
      // @ts-ignore
      window.__PINNACLE_PDF_STATE__.annotationRectInput = "100, 100, 200, 150";
    });

    // Enter field name
    await page.fill('input#new-field-name', 'ClientName');

    // Add field
    await page.click('button:has-text("Add")');

    // Check if field is in the list
    await expect(page.getByText('ClientName')).toBeVisible();

    // Save
    await page.click('button:has-text("Save & Export Document")');

    // Check status
    await expect(page.getByTestId('status-message')).toContainText('Form saved successfully');
  });

  test('should trigger AI detection', async ({ page }) => {
    await page.click('button[title="Forms"]', { force: true });
    await page.click('button:has-text("Builder Mode")');

    // Select PDF
    await page.click('button:has-text("Select PDF Form")');

    // Click AI Detect
    await page.click('button:has-text("✨ AI Detect")');

    // Check status
    await expect(page.getByTestId('status-message')).toContainText('AI detected');
  });
});
