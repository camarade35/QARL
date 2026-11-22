/**
 * @param {{ tport?: number | null, gamePort?: number | null, port?: number | null } | null} details
 * @param {{ source?: string, game_port?: number | null, port?: number | null }} server
 * @returns {number | null}
 */
export function getServerGamePort(details, server) {
  if (server.source === 'kunos') {
    return details?.tport || server.game_port || details?.gamePort || null;
  }
  return details?.tport || details?.gamePort || details?.port || server.port || null;
}
