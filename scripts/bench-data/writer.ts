const encoder = new TextEncoder();

const BYTE_NEWLINE = 0x0a; // "\n"
const BYTES_NEWLINE = encoder.encode("\n");
const BYTE_ZERO = 0x30; // "0"
const BYTE_MINUS = 0x2d; // "-"

/** Longest decimal form of a safe integer, "-9007199254740991". */
const MAX_INT_BYTES = 17;

/** Longest run that `writeBytes` copies manually rather than with `set`. */
const SHORT_RUN_BYTES = 16;

/**
 * Deepest level whose line prefix is held in the prefix table. */
export const MAX_INDENT_CACHE_SIZE = 256;

export class Writer {
  readonly #target: WriterResource;
  readonly #indentUnitBytes: Uint8Array; // bytes of the indentation unit
  readonly #buffer: Uint8Array;
  /**
   * The line prefix for each level, indexed by level.
   * Every entry is a view into one shared repeated-indent buffer.
   */
  readonly #linePrefixes: readonly Uint8Array[];
  readonly #maxLineWidth: number; // Target width a line should stay within
  #level = 0; // The current indentation level
  #budget: number; // Approximate bytes left before writers should stop
  #buffered = 0; // Bytes currently sitting in #buffer
  #lineWidth = 0; // Bytes written on the current line
  #closed = false;

  constructor(
    target: WriterResource,
    indentUnit = "  ", // string used for 1 level of indentation
    maxLineWidth = Infinity, // target width a line should stay within
    budget = Infinity,
    bufferSize = 64 * 1024, // 64 kB
  ) {
    this.#target = target;
    this.#indentUnitBytes = encoder.encode(indentUnit);
    this.#linePrefixes = linePrefixes(this.#indentUnitBytes);
    this.#maxLineWidth = maxLineWidth;
    this.#budget = budget;
    this.#buffer = new Uint8Array(bufferSize);
  }

  /** Increases the indentation level by one. */
  indent(): this {
    this.#level++;
    return this;
  }

  /** Decreases the indentation level by one, never below zero. */
  dedent(): this {
    if (this.#level > 0) {
      this.#level--;
    }
    return this;
  }

  /** Appends a string to the current line. */
  writeString(text: string): this {
    // Three bytes per UTF-16 code unit is the worst case in UTF-8.
    if (3 * text.length > this.#buffer.length - this.#buffered) {
      this.flush();
      if (3 * text.length > this.#buffer.length) {
        // Longer than the buffer, so it cannot be buffered at all.
        this.writeBytes(encoder.encode(text));
        return this;
      }
    }

    const { written } = encoder.encodeInto(
      text,
      this.#buffer.subarray(this.#buffered),
    );
    this.#buffered += written;
    this.#budget -= written;
    this.#lineWidth += written;
    return this;
  }

  /** Append bytes to the current line. */
  writeBytes(data: Uint8Array): void {
    if (data.length === 0) {
      return;
    }

    // Flush the buffer if data doesn't fit in the available free space
    if (data.length > this.#buffer.length - this.#buffered) {
      this.flush();

      // Write directly if data is larger than the entire buffer
      if (data.length > this.#buffer.length) {
        this.#budget -= data.length;
        this.#lineWidth += data.length;
        this.#target.writeSync(data);
        return;
      }
    }

    // Append the data to the buffer.
    const buffer = this.#buffer;
    const at = this.#buffered;
    if (data.length <= SHORT_RUN_BYTES) {
      // Manually copy small runs
      for (let i = 0; i < data.length; i++) {
        buffer[at + i] = data[i];
      }
    } else {
      buffer.set(data, at);
    }
    this.#buffered = at + data.length;
    this.#budget -= data.length;
    this.#lineWidth += data.length;
  }

  /** Appends a single byte to the current line. */
  writeByte(byte: number): this {
    if (this.#buffered === this.#buffer.length) {
      this.flush();
    }

    this.#buffer[this.#buffered] = byte;
    this.#buffered++;
    this.#budget--;
    this.#lineWidth++;
    return this;
  }

  /** Appends an ASCII string to the current line. */
  writeAscii(text: string): this {
    const length = text.length;

    if (length > this.#buffer.length - this.#buffered) {
      this.flush();
      if (length > this.#buffer.length) {
        // Longer than the buffer, so it cannot be buffered at all.
        this.writeBytes(encoder.encode(text));
        return this;
      }
    }

    const buffer = this.#buffer;
    const at = this.#buffered;
    for (let i = 0; i < length; i++) {
      buffer[at + i] = text.charCodeAt(i);
    }
    this.#buffered = at + length;
    this.#budget -= length;
    this.#lineWidth += length;
    return this;
  }

  /** Appends the decimal form of `value`, which must be a safe integer. */
  writeInt(value: number): this {
    // Reserve enough room so every store below is unconditional.
    if (this.#buffer.length - this.#buffered < MAX_INT_BYTES) {
      this.flush();
    }

    let magnitude = value;
    let negative = false;
    if (magnitude < 0) {
      negative = true;
      magnitude = -magnitude;
      this.#buffer[this.#buffered] = BYTE_MINUS;
      this.#buffered++;
    }

    // Count the digits first, so the digits can be filled in backwards from the end of the number.
    let digits = 1;
    for (
      let rest = Math.floor(magnitude / 10);
      rest > 0;
      rest = Math.floor(rest / 10)
    ) {
      digits++;
    }

    this.#buffered += digits;
    let at = this.#buffered;
    while (magnitude >= 10) {
      const rest = Math.floor(magnitude / 10);
      this.#buffer[--at] = BYTE_ZERO + (magnitude - rest * 10);
      magnitude = rest;
    }
    this.#buffer[--at] = BYTE_ZERO + magnitude;

    const written = digits + (negative ? 1 : 0);
    this.#budget -= written;
    this.#lineWidth += written;
    return this;
  }

  /** Ends the current line and indents the next one to the current level. */
  newline(): this {
    this.#lineWidth = 0;

    if (this.#level <= MAX_INDENT_CACHE_SIZE) {
      this.writeBytes(this.#linePrefixes[this.#level]);
      return this;
    }

    // fallback: Deeper than the prefix buffer holds. Only reachable for nesting far beyond any real recipe.
    this.writeBytes(BYTES_NEWLINE);
    for (let i = 0; i < this.#level; i++) {
      this.writeBytes(this.#indentUnitBytes);
    }
    return this;
  }

  /** Writes any buffered bytes to the target. */
  flush(): void {
    if (this.#buffered === 0) {
      return;
    }
    // Bytes `[0, #buffered)` hold data, so pass a view of just those
    // rather than the whole buffer.
    this.#target.writeSync(this.#buffer.subarray(0, this.#buffered));
    this.#buffered = 0;
  }

  /** Approximate bytes still available, possibly negative once spent. */
  get budget(): number {
    return this.#budget;
  }

  /** True while the current line still fits its target width. */
  get hasLineBudget(): boolean {
    return this.#lineWidth <= this.#maxLineWidth;
  }

  /** True while the byte budget is not yet spent. */
  get hasBudget(): boolean {
    return this.#budget > 0;
  }

  /** Flushes the output and closes the target, if it is closeable. */
  close(): void {
    if (this.#closed) {
      return;
    }
    this.#closed = true;
    this.flush();
    this.#target.close?.();
  }
}

export interface WriterResource {
  /** Writes `data` to the underlying resource. */
  writeSync(data: Uint8Array): number;
  /** Releases the resource. */
  close?(): void;
}

/** Builds `[\n, \n+indent, \n+indent*2, ...]` up to `MAX_INDENT_CACHE_SIZE`. */
function linePrefixes(indent: Uint8Array): readonly Uint8Array[] {
  const buffer = new Uint8Array(1 + MAX_INDENT_CACHE_SIZE * indent.length);
  buffer[0] = BYTE_NEWLINE;
  for (let at = 1; at < buffer.length; at += indent.length) {
    buffer.set(indent, at);
  }

  const byLevel: Uint8Array[] = new Array(MAX_INDENT_CACHE_SIZE + 1);
  for (let level = 0; level <= MAX_INDENT_CACHE_SIZE; level++) {
    byLevel[level] = buffer.subarray(0, 1 + level * indent.length);
  }

  return byLevel;
}
