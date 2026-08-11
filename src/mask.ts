export interface MaskViewState {
  id: string;
  name: string;
  color: string;
  opacity: number;
  locked: boolean;
  visible: boolean;
  trayReady: boolean;
}

export type ResizeDirection =
  | 'East'
  | 'North'
  | 'NorthEast'
  | 'NorthWest'
  | 'South'
  | 'SouthEast'
  | 'SouthWest'
  | 'West';

export const resizeHandles: ReadonlyArray<{ className: string; direction: ResizeDirection }> = [
  { className: 'north', direction: 'North' },
  { className: 'south', direction: 'South' },
  { className: 'west', direction: 'West' },
  { className: 'east', direction: 'East' },
  { className: 'north-west', direction: 'NorthWest' },
  { className: 'north-east', direction: 'NorthEast' },
  { className: 'south-west', direction: 'SouthWest' },
  { className: 'south-east', direction: 'SouthEast' },
];

export function maskColorToRgba(color: string, opacity: number): string {
  const normalized = /^#[0-9A-Fa-f]{6}$/.test(color) ? color : '#000000';
  const hex = normalized.slice(1);
  const red = Number.parseInt(hex.slice(0, 2), 16);
  const green = Number.parseInt(hex.slice(2, 4), 16);
  const blue = Number.parseInt(hex.slice(4, 6), 16);
  const alpha = Math.min(100, Math.max(10, opacity)) / 100;
  return `rgba(${red}, ${green}, ${blue}, ${alpha})`;
}
