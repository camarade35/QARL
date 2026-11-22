import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import { existsSync } from 'node:fs';
import { getFlagUrl } from '../src/lib/utils/assets.js';

test('flag URLs for lobby country codes point to existing assets', () => {
  for (const country of ['sa', 'th', 'tw', 'id', 'is', 'vn', 'kr']) {
    const url = getFlagUrl(country);
    assert.ok(url, `Missing flag for ${country}`);
    assert.ok(existsSync(`static${url}`), `Missing asset for ${country}: ${url}`);
  }
});

test('countries without flag assets use the fallback instead of a broken URL', () => {
  for (const country of ['pk', 'bd', 'eg', 'ke']) {
    assert.equal(getFlagUrl(country), null);
    assert.equal(getFlagUrl(country, '/fallback.png'), '/fallback.png');
  }
});
