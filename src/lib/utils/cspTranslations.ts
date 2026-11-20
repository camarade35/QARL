export function getCspOptionTranslationKeys(key: string, label: string): string[] {
  const normalizedLabel = label
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/[^a-z0-9]+/gi, '_')
    .replace(/^_+|_+$/g, '')
    .toLowerCase();

  return [...new Set([key, label, normalizedLabel].filter(Boolean))]
    .map(candidate => `csp.option_labels.${candidate}`);
}

export function normalizeCspSearchText(value: string): string {
  return value
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase();
}

export function getCspSectionTranslationKey(section: string): string {
  return `csp.section_names.${section}`;
}
