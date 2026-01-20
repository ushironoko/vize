import { test, expect } from '@playwright/test';

const PRESETS = [
  'Overview',
  'Reactivity Loss',
  'Setup Context',
  'Reference Escape',
  'Provide/Inject Tree',
  'Fallthrough Attrs',
];

for (const preset of PRESETS) {
  test(`Debug ${preset} preset`, async ({ page }) => {
    // Capture console logs for debugging
    page.on('console', msg => {
      const text = msg.text();
      if (text.includes('[DEBUG]') && text.includes('offset=')) {
        console.log('BROWSER:', text);
      }
      // Also capture WASM DEBUG logs
      if (text.includes('[WASM DEBUG]')) {
        console.log('WASM:', text);
      }
    });

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Click CF tab
    await page.click('text=CF');
    await page.waitForTimeout(1000);

    // Click preset
    await page.click(`text=${preset}`);
    await page.waitForTimeout(2000);

    // Take screenshot
    const safeName = preset.replace(/[^a-zA-Z0-9]/g, '-').toLowerCase();
    await page.screenshot({ path: `test-results/${safeName}.png`, fullPage: true });

    // Get diagnostics
    const diagPanel = page.locator('.diagnostics-pane');
    const diagText = await diagPanel.first().textContent();
    console.log(`\n=== ${preset} DIAGNOSTICS ===`);
    console.log(diagText);

    expect(true).toBeTruthy();
  });
}
