const DATE_FORMAT = new Intl.DateTimeFormat('es-ES', {
  dateStyle: 'long',
  timeZone: 'UTC',
});

export function formatDate(value) {
  if (!value) return 'fecha desconocida';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return 'fecha desconocida';
  return DATE_FORMAT.format(date);
}

export function formatBytes(value) {
  const bytes = Number(value);
  if (!Number.isFinite(bytes) || bytes <= 0) return 'tamaño desconocido';
  const units = ['B', 'KiB', 'MiB', 'GiB'];
  let amount = bytes;
  let unit = 0;
  while (amount >= 1024 && unit < units.length - 1) {
    amount /= 1024;
    unit += 1;
  }
  const decimals = unit === 0 || amount >= 100 ? 0 : 1;
  return `${amount.toFixed(decimals)} ${units[unit]}`;
}

export function shortChecksum(value) {
  const hash = String(value ?? '');
  if (!/^[a-fA-F0-9]{64}$/.test(hash)) return 'checksum desconocido';
  return hash.slice(0, 12);
}
