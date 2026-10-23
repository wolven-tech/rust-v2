const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');

async function main() {
  const browser = await chromium.launch({ headless: true });
  const deadline = setTimeout(() => { void browser.close(); }, 60000);
  const assets = path.resolve(__dirname, '../assets');
  try {
    for (const viewport of [{ width: 1280, height: 800 }, { width: 375, height: 667 }]) {
      const page = await browser.newPage({ viewport });
      await page.setContent('<main style="height:1800px">Example page</main><footer style="transform:translateZ(0);content-visibility:auto"><a href="#home">Home</a></footer>');
      await page.addStyleTag({ content: fs.readFileSync(path.join(assets, 'consent-footer.css'), 'utf8') });
      await page.addScriptTag({ content: fs.readFileSync(path.join(assets, 'consent-footer.js'), 'utf8') });
      await page.evaluate(() => {
        window.choices = [];
        window.consent = window.Rv2ConsentFooter.mount({
          id: 'test-consent', choice: null, privacyHref: '/privacy',
          onChoice: (choice) => window.choices.push(choice)
        });
      });
      const panel = page.locator('#test-consent');
      assert.equal(await panel.evaluate((node) => node.parentElement === document.body), true);
      const bounds = await panel.boundingBox();
      assert.equal(Math.round(bounds.y + bounds.height), viewport.height);
      assert.ok(bounds.width <= viewport.width);
      assert.ok(bounds.height <= viewport.height * .65 + 1);
      assert.equal(await panel.getByRole('link', { name: 'Privacy policy' }).getAttribute('href'), '/privacy');
      await panel.getByRole('button', { name: 'Dismiss analytics preferences' }).click();
      assert.equal(await panel.isVisible(), false);
      assert.deepEqual(await page.evaluate(() => window.choices), ['declined']);
      assert.equal(await page.getByRole('button', { name: 'Privacy preferences', exact: true }).evaluate((node) => node === document.activeElement), true);
      await page.getByRole('button', { name: 'Privacy preferences', exact: true }).click();
      await panel.getByRole('button', { name: 'Allow analytics' }).click();
      assert.equal(await panel.isVisible(), false);
      assert.deepEqual(await page.evaluate(() => window.choices), ['declined', 'accepted']);
      await page.getByRole('button', { name: 'Privacy preferences', exact: true }).click();
      await panel.getByRole('button', { name: 'Dismiss analytics preferences' }).click();
      assert.deepEqual(await page.evaluate(() => window.choices), ['declined', 'accepted']);
      await page.getByRole('button', { name: 'Privacy preferences', exact: true }).click();
      await panel.getByRole('button', { name: 'Withdraw consent' }).click();
      await page.evaluate(() => window.consent.update({ blocked: true }));
      assert.equal(await panel.isVisible(), false);
      await page.getByRole('button', { name: 'Privacy preferences', exact: true }).click();
      assert.equal(await panel.getByRole('button', { name: 'Allow analytics' }).isDisabled(), true);
      assert.match(await panel.textContent(), /browser requests privacy/);
      await panel.getByRole('button', { name: 'Keep analytics off' }).click();
      await page.evaluate(() => window.consent.update({ blocked: false, storageUnavailable: true }));
      await page.getByRole('button', { name: 'Privacy preferences', exact: true }).click();
      assert.equal(await panel.getByRole('button', { name: 'Allow analytics' }).isDisabled(), false);
      await panel.getByRole('button', { name: 'Allow analytics' }).click();
      await page.getByRole('button', { name: 'Privacy preferences', exact: true }).click();
      assert.match(await panel.textContent(), /Analytics are on/);
      assert.match(await panel.textContent(), /applies to this visit/);
      await page.evaluate(() => window.consent.update({ choice: 'declined' }));
      await page.evaluate(() => window.consent.update({ choice: 'accepted', storageUnavailable: false }));
      assert.equal(await panel.isVisible(), false);
      await page.evaluate(() => {
        window.previous = window.consent;
        window.consent = window.Rv2ConsentFooter.mount({ id: 'test-consent', choice: 'declined' });
        window.previous.destroy();
      });
      assert.equal(await panel.count(), 1);
      assert.equal(await page.getByRole('button', { name: 'Privacy preferences', exact: true }).count(), 1);
      assert.equal(await panel.isVisible(), false);
      await page.evaluate(() => window.consent.destroy());
      assert.equal(await panel.count(), 0);
      assert.equal(await page.getByRole('button', { name: 'Privacy preferences', exact: true }).count(), 0);
      await page.close();
      process.stdout.write(`Consent footer ${viewport.width}px: lifecycle, fixed geometry and privacy controls passed\n`);
    }
  } finally {
    clearTimeout(deadline);
    await browser.close();
  }
}
main().catch((error) => { process.stderr.write(`${error.stack}\n`); process.exitCode = 1; });
