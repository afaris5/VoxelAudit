import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';
import { deflateSync } from 'node:zlib';
import { Buffer } from 'node:buffer';

function crc32(data) {
  let crc = 0xffffffff;
  for (const byte of data) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit++)
      crc = (crc >>> 1) ^ (0xedb88320 & -(crc & 1));
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const name = Buffer.from(type);
  const result = Buffer.alloc(12 + data.length);
  result.writeUInt32BE(data.length);
  name.copy(result, 4);
  data.copy(result, 8);
  result.writeUInt32BE(crc32(Buffer.concat([name, data])), 8 + data.length);
  return result;
}

function png(size, pixel) {
  const header = Buffer.alloc(13);
  header.writeUInt32BE(size, 0);
  header.writeUInt32BE(size, 4);
  header[8] = 8;
  header[9] = 6;
  const rows = Buffer.alloc(size * (1 + size * 4));
  for (let y = 0; y < size; y++)
    for (let x = 0; x < size; x++) {
      const offset = y * (1 + size * 4) + 1 + x * 4;
      pixel(x, y).forEach((value, index) => {
        rows[offset + index] = value;
      });
    }
  return Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    chunk('IHDR', header),
    chunk('IDAT', deflateSync(rows)),
    chunk('IEND', Buffer.alloc(0)),
  ]);
}

function write(path, data) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, data);
}

const icon = png(256, (x, y) => {
  const px = x / 4;
  const py = y / 4;
  const inside =
    px >= 12 &&
    px <= 52 &&
    py >= 8 + Math.abs(px - 32) * 0.55 &&
    py <= 56 - Math.abs(px - 32) * 0.6;
  if (!inside) return [28, 34, 37, 255];
  const line =
    Math.abs(py - (31 - Math.abs(px - 32) * 0.6)) < 1.5 ||
    (Math.abs(px - 32) < 1.5 && py >= 31);
  return line ? [28, 34, 37, 255] : [157, 201, 164, 255];
});
write('src-tauri/icons/icon.png', icon);
const ico = Buffer.alloc(22);
ico.writeUInt16LE(1, 2);
ico.writeUInt16LE(1, 4);
ico.writeUInt16LE(1, 10);
ico.writeUInt16LE(32, 12);
ico.writeUInt32LE(icon.length, 14);
ico.writeUInt32LE(22, 18);
write('src-tauri/icons/icon.ico', Buffer.concat([ico, icon]));

for (const project of ['valid-pack', 'broken-pack']) {
  write(
    `fixtures/${project}/assets/atelier/textures/block/slate.png`,
    png(16, (x, y) => {
      const value = ((x * 7 + y * 13) % 5) * 6;
      return [74 + value, 100 + value, 89 + value, 255];
    }),
  );
}
write(
  'fixtures/valid-pack/assets/atelier/textures/item/crystal.png',
  png(16, (x, y) =>
    Math.abs(x - 7.5) + Math.abs(y - 7.5) < 6
      ? [133, 208, 187, 255]
      : [0, 0, 0, 0],
  ),
);
write(
  'fixtures/broken-pack/assets/atelier/textures/block/odd.png',
  png(15, (x, y) => [x * 15, y * 15, 100, 255]),
);
