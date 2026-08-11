import { describe, expect, it } from 'vitest';
import { maskColorToRgba, resizeHandles } from './mask';

describe('resize handles', () => {
  it('maps every edge and corner to one unique native direction', () => {
    expect(resizeHandles).toHaveLength(8);
    expect(new Set(resizeHandles.map((handle) => handle.direction))).toEqual(
      new Set(['North', 'South', 'East', 'West', 'NorthEast', 'NorthWest', 'SouthEast', 'SouthWest']),
    );
  });
});

describe('maskColorToRgba', () => {
  it('converts a validated color and opacity', () => {
    expect(maskColorToRgba('#1D4ED8', 80)).toBe('rgba(29, 78, 216, 0.8)');
  });

  it('uses safe bounds for invalid values', () => {
    expect(maskColorToRgba('invalid', 0)).toBe('rgba(0, 0, 0, 0.1)');
    expect(maskColorToRgba('#FFFFFF', 120)).toBe('rgba(255, 255, 255, 1)');
  });
});
