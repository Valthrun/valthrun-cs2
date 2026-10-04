/*
 * One-time helper that computes the tight content bounds of each rush room image.
 *
 * The room squares declared in rush_001.txt overlap slightly, but the actual
 * radar content inside them does not. Scanning the images gives exact bounds so
 * a world position resolves to a single room.
 *
 * Usage (from radar/web):
 *     node scripts/gen-volume-bounds.js
 *
 * It writes src/map-info/rush/rush_001/volumes.json.
 */

const fs = require("fs");
const path = require("path");
const zlib = require("zlib");

const MAP_DIR = path.resolve(__dirname, "..", "src", "map-info", "rush", "rush_001");
const OVERVIEW = path.resolve(__dirname, "..", "src", "map-info", "resource", "overviews", "rush_001.txt");
const OUTPUT = path.join(MAP_DIR, "volumes.json");

const parseVolumes = (text) => {
    const volumes = [];
    const volumeBlocks = text.split(/"(room\d+|convoy|roomparty)"\s*\{/).slice(1);
    for (let i = 0; i < volumeBlocks.length; i += 2) {
        const name = volumeBlocks[i];
        const body = volumeBlocks[i + 1];
        const posX = Number(/pos_x"?\s*"?(-?[\d.]+)/.exec(body)[1]);
        const posY = Number(/pos_y"?\s*"?(-?[\d.]+)/.exec(body)[1]);
        const scale = Number(/scale"?\s*"?([\d.]+)/.exec(body)[1]);
        volumes.push({ name, pos_x: posX, pos_y: posY, scale });
    }
    return volumes;
};

/* Reads a PNG and returns the bounding box of non-black opaque pixels. */
const contentBounds = (file) => {
    const buffer = fs.readFileSync(file);
    let offset = 8;
    let width = 0;
    let height = 0;
    const idat = [];
    while (offset < buffer.length) {
        const length = buffer.readUInt32BE(offset);
        const type = buffer.toString("ascii", offset + 4, offset + 8);
        const data = buffer.subarray(offset + 8, offset + 8 + length);
        if (type === "IHDR") {
            width = data.readUInt32BE(0);
            height = data.readUInt32BE(4);
        } else if (type === "IDAT") {
            idat.push(data);
        }
        offset += 12 + length;
    }

    const raw = zlib.inflateSync(Buffer.concat(idat));
    const stride = width * 4 + 1;
    let minX = width;
    let minY = height;
    let maxX = -1;
    let maxY = -1;
    const prev = Buffer.alloc(width * 4);

    for (let y = 0; y < height; y++) {
        const filter = raw[y * stride];
        const line = Buffer.from(raw.subarray(y * stride + 1, y * stride + 1 + width * 4));
        for (let x = 0; x < width; x++) {
            for (let c = 0; c < 4; c++) {
                const i = x * 4 + c;
                const a = x > 0 ? line[i - 4] : 0;
                const b = prev[i];
                const cc = x > 0 ? prev[i - 4] : 0;
                let value = line[i];
                if (filter === 1) value += a;
                else if (filter === 2) value += b;
                else if (filter === 3) value += (a + b) >> 1;
                else if (filter === 4) {
                    const p = a + b - cc;
                    const pa = Math.abs(p - a);
                    const pb = Math.abs(p - b);
                    const pc = Math.abs(p - cc);
                    value += pa <= pb && pa <= pc ? a : pb <= pc ? b : cc;
                }
                line[i] = value & 0xff;
            }
        }

        for (let x = 0; x < width; x++) {
            const i = x * 4;
            if (line[i] > 8 || line[i + 1] > 8 || line[i + 2] > 8) {
                if (x < minX) minX = x;
                if (x > maxX) maxX = x;
                if (y < minY) minY = y;
                if (y > maxY) maxY = y;
            }
        }

        line.copy(prev);
    }

    return { minX, minY, maxX, maxY };
};

const text = fs.readFileSync(OVERVIEW, "utf8");
const volumes = parseVolumes(text);
const result = {};

for (const volume of volumes) {
    const file = path.join(MAP_DIR, `map_${volume.name}.png`);
    if (!fs.existsSync(file)) {
        console.warn(`missing image for volume ${volume.name}`);
        continue;
    }

    const bounds = contentBounds(file);
    const toWorldX = (px) => volume.pos_x + px * volume.scale;
    const toWorldY = (py) => volume.pos_y - py * volume.scale;
    result[volume.name] = {
        /* Pixel bounds within the room image, inclusive. */
        minX: bounds.minX,
        minY: bounds.minY,
        maxX: bounds.maxX,
        maxY: bounds.maxY,
        /* Tight world-space bounds of the room content. */
        worldMinX: Math.round(toWorldX(bounds.minX)),
        worldMaxX: Math.round(toWorldX(bounds.maxX + 1)),
        worldMinY: Math.round(toWorldY(bounds.maxY + 1)),
        worldMaxY: Math.round(toWorldY(bounds.minY)),
    };
}

fs.writeFileSync(OUTPUT, JSON.stringify(result, null, 4) + "\n");
console.log(`wrote ${Object.keys(result).length} volume bounds to ${path.relative(process.cwd(), OUTPUT)}`);
