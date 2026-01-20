import { test, expect } from '@playwright/test';

test('Debug CroquisCF offset', async ({ page }) => {
  // Capture ALL console logs
  page.on('console', msg => {
    const text = msg.text();
    console.log(`CONSOLE [${msg.type()}]:`, text);
  });

  // Capture page errors
  page.on('pageerror', err => {
    console.log('PAGE ERROR:', err.message);
  });

  await page.goto('/');
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(2000);

  // Click CroquisCF button to enable cross-file analysis
  await page.click('button:has-text("CroquisCF")');
  await page.waitForTimeout(1000);

  // Click "Reactivity Loss" preset from sidebar
  await page.click('text=Reactivity Loss');
  await page.waitForTimeout(3000);

  // Take screenshot
  await page.screenshot({ path: 'test-results/reactivity-loss.png', fullPage: true });

  // Get any visible diagnostics text
  const bodyText = await page.locator('body').textContent();
  if (bodyText && bodyText.includes('offset')) {
    console.log('Found offset info in body');
  }

  expect(true).toBeTruthy();
});
