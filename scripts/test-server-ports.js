import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import { getServerGamePort } from '../src/lib/utils/serverPorts.js';

test('featured and recent servers use the advertised TCP port instead of UDP or HTTP', () => {
  const details = { port: 9603, tport: 9604, cport: 9605 };
  assert.equal(getServerGamePort(details, { source: 'featured', port: 9605 }), 9604);
  assert.equal(getServerGamePort(details, { port: 9605 }), 9604);
});

test('preserves existing port fallbacks and does not use a Kunos UDP or HTTP port', () => {
  assert.equal(getServerGamePort({ port: 9603 }, { port: 9605 }), 9603);
  assert.equal(getServerGamePort(null, { port: 9605 }), 9605);
  assert.equal(getServerGamePort({ port: 9603 }, { source: 'kunos', game_port: 9604, port: 9605 }), 9604);
  assert.equal(getServerGamePort({ port: 9603 }, { source: 'kunos', port: 9605 }), null);
});
