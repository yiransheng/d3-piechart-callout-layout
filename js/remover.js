export class Interval {
  constructor(weight, lower, upper) {
    this._weight = weight;
    this._lower = lower;
    this._upper = upper;
  }
  encode(arr, offset) {
    arr[offset] = this._lower;
    arr[offset + 1] = this._upper;
    arr[offset + 2] = Math.floor(this._weight);

    return offset + 3;
  }
}
Interval.sizeOf = () => 3;

export function fromModule(module) {

  return function(items, accessor) {
    const n = items.length;
    const labels = new Float64Array(Interval.sizeOf() * n);
    const keep = new Uint8Array(n);

    let i = 0;
    for (const item of items) {
      const interval = accessor(item);
      i = interval.encode(labels, i);
    }

    module.remove_overlapping(labels, keep);

    const result = [];
    for (let i = 0; i < keep.length; i++) {
      if (keep[i] > 0) {
        result.push(items[i]);
      }
    }
    return result;
  };
}
