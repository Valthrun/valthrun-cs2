/*
 * Finding Maps Source
 * Find Counter-Strike Global Offensive\game\csgo\pak01_dir.vpk
 * Open with Source Viewer ( https://github.com/ValveResourceFormat/ValveResourceFormat )
 * Locate resource/overviews / Locate panorama/images/overheadmaps
 * */
import { KeyValuesNode, parseKeyValues } from "./key-values";


export type VerticalSection = {
    name: "default" | "lower",
    altitudeMax: number,
    altitudeMin: number,
}

export type MapStyle = {
    name: string,
    map: {
        default: string,
        lower?: string
    }
}

export type MapVolume = {
    name: string;

    pos_x: number,
    pos_y: number,
    scale: number,

    image: string,
    lowerImage?: string,

    /* Tight world-space bounds of the room content. */
    worldMinX: number,
    worldMaxX: number,
    worldMinY: number,
    worldMaxY: number,
}

export type LoadedMap = {
    mapName: string;
    displayName: string;

    pos_x: number,
    pos_y: number,
    scale: number,

    /* Pixel width of the official radar image; the world extent is scale * imageWidth. */
    imageWidth: number,

    verticalSections: VerticalSection[],
    mapStyles: MapStyle[],
    volumes: MapVolume[]
};

/* Map coordinates and radar images are discovered from the layout */
const kMapImageContext = import.meta.webpackContext(".", {
    recursive: true,
    regExp: /^\.\/[^/]+\/[^/]+\/map_[a-z0-9_]+\.png$/,
    mode: "sync",
});

const kOverviewContext = import.meta.webpackContext("./resource/overviews", {
    recursive: false,
    regExp: /^\.\/[^/]+\.txt$/,
    mode: "sync",
});

const kVolumeBoundsContext = import.meta.webpackContext(".", {
    recursive: true,
    regExp: /^\.\/[^/]+\/[^/]+\/volumes\.json$/,
    mode: "sync",
});

/** Turns a map folder name such as "de_dust2" into a display name such as "Dust2". */
const displayNameFromMapName = (mapName: string): string => {
    const suffix = mapName.split("_").slice(1).join(" ");
    const base = suffix.length > 0 ? suffix : mapName;
    return base
        .split(" ")
        .filter((part) => part.length > 0)
        .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
        .join(" ");
};

const unwrapDefault = (module: any): any => (module && module.default !== undefined ? module.default : module);

type ImageGroups = {
    styles: Record<string, { default?: string, lower?: string }>,
    volumes: Record<string, string>,
};

const buildMapImages = (): Record<string, ImageGroups> => {
    const images: Record<string, ImageGroups> = {};

    for (const request of kMapImageContext.keys()) {
        const segments = request.split("/");
        const mapName = segments[segments.length - 2];
        const fileName = segments[segments.length - 1];

        const match = /^map_([a-z0-9_]+?)\.png$/.exec(fileName);
        if (!match) {
            continue;
        }

        let name = match[1];
        const url: string = unwrapDefault(kMapImageContext(request));
        const groups = images[mapName] ?? (images[mapName] = { styles: {}, volumes: {} });

        let isLower = false;
        if (name.endsWith("_lower")) {
            name = name.slice(0, -"_lower".length);
            isLower = true;
        }

        if (name === "default" || name === "simple") {
            const style = groups.styles[name] ?? (groups.styles[name] = {});
            if (isLower) {
                style.lower = url;
            } else {
                style.default = url;
            }
        } else {
            groups.volumes[name] = url;
        }
    }

    return images;
};

const kMapImages = buildMapImages();

const kVolumeBounds: Record<string, Record<string, any>> = {};
for (const request of kVolumeBoundsContext.keys()) {
    const segments = request.split("/");
    kVolumeBounds[segments[segments.length - 2]] = unwrapDefault(kVolumeBoundsContext(request));
}

/*
 * The world extent of a radar image is scale * its pixel width. Official maps
 * use a 1024px overview while RadGen-generated maps often use 2048px, and
 * scale is per pixel of that image. The width is read from the image itself so
 * swapping or adding a radar image needs no generated metadata.
 */
const kDefaultImageWidth = 1024;
const probeImageWidth = (url: string | undefined): Promise<number> => {
    return new Promise((resolve) => {
        if (!url || typeof Image === "undefined") {
            resolve(kDefaultImageWidth);
            return;
        }

        const image = new Image();
        image.onload = () => resolve(image.naturalWidth || kDefaultImageWidth);
        image.onerror = () => resolve(kDefaultImageWidth);
        image.src = url;
    });
};

const kOverviews: Record<string, KeyValuesNode> = {};
for (const request of kOverviewContext.keys()) {
    const fileName = request.split("/").pop() as string;
    const mapName = fileName.replace(/\.txt$/, "");
    kOverviews[mapName] = parseKeyValues(unwrapDefault(kOverviewContext(request)) as string);
}

const numberOr = (value: string | KeyValuesNode | undefined, fallback: number): number => {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : fallback;
};

const rootValues = (overview: KeyValuesNode): KeyValuesNode => {    for (const key of Object.keys(overview)) {
        const value = overview[key];
        if (typeof value !== "string") {
            return value;
        }
    }

    return overview;
};

const buildVerticalSections = (overview: KeyValuesNode): VerticalSection[] => {
    const sectionsNode = overview["verticalsections"] as KeyValuesNode | undefined;
    const raw: { altitudeMax: number, altitudeMin: number }[] = [];

    if (sectionsNode) {
        for (const name of Object.keys(sectionsNode)) {
            const section = sectionsNode[name] as KeyValuesNode;
            const altitudeMax = numberOr(section["altitudemax"], NaN);
            const altitudeMin = numberOr(section["altitudemin"], NaN);
            if (!Number.isFinite(altitudeMax) || !Number.isFinite(altitudeMin)) {
                continue;
            }

            raw.push({ altitudeMax, altitudeMin });
        }
    }

    if (raw.length === 0) {
        return [{ name: "default", altitudeMax: 10000, altitudeMin: -10000 }];
    }

    /*
     * The UI only has two level slots (default and lower). Maps name their
     * sections differently (nuke "lower", boulder "higher1"), so assign by
     * order rather than by name.
     */
    if (raw.length > 2) {
        console.warn(`Overview has ${raw.length} vertical sections; only the first two are used.`);
    }

    return raw.slice(0, 2).map((section, index) => ({
        name: index === 0 ? "default" : "lower",
        altitudeMax: section.altitudeMax,
        altitudeMin: section.altitudeMin,
    }));
};

const buildVolumes = (overview: KeyValuesNode, mapImages: ImageGroups | undefined, volumeBounds: Record<string, any>): MapVolume[] => {
    const volumes: MapVolume[] = [];
    if (!mapImages) {
        return volumes;
    }

    const volumesNode = overview["volumes"] as KeyValuesNode | undefined;
    if (!volumesNode) {
        return volumes;
    }

    for (const name of Object.keys(volumesNode)) {
        const volume = volumesNode[name] as KeyValuesNode;
        const image = mapImages.volumes[name];
        if (!image) {
            continue;
        }

        const bounds = volumeBounds[name];
        volumes.push({
            name,
            pos_x: numberOr(volume["pos_x"], 0),
            pos_y: numberOr(volume["pos_y"], 0),
            scale: numberOr(volume["scale"], 1),
            image,
            lowerImage: mapImages.volumes[`${name}_lower`],
            worldMinX: bounds?.worldMinX ?? Number.NEGATIVE_INFINITY,
            worldMaxX: bounds?.worldMaxX ?? Number.POSITIVE_INFINITY,
            worldMinY: bounds?.worldMinY ?? Number.NEGATIVE_INFINITY,
            worldMaxY: bounds?.worldMaxY ?? Number.POSITIVE_INFINITY,
        });
    }

    return volumes;
};

const buildMapStyles = (mapImages: ImageGroups | undefined): MapStyle[] => {
    const styles: MapStyle[] = [];
    if (!mapImages) {
        return styles;
    }

    const order = ["Official", "SimpleRadar"];
    const fallbackLower = mapImages.styles["default"]?.lower;

    for (const styleName of order) {
        const image = mapImages.styles[styleName === "Official" ? "default" : "simple"];
        if (!image || !image.default) {
            continue;
        }

        const map: MapStyle["map"] = { default: image.default };
        const lower = image.lower ?? fallbackLower;
        if (lower) {
            map.lower = lower;
        }

        styles.push({ name: styleName, map });
    }

    return styles;
};

const buildMapRegistry = (): Record<string, () => Promise<LoadedMap>> => {
    const registry: Record<string, () => Promise<LoadedMap>> = {};

    for (const mapName of Object.keys(kMapImages)) {
        const overview = kOverviews[mapName];
        if (!overview) {
            console.warn(`Map "${mapName}" has no resource/overviews/${mapName}.txt; skipping.`);
            continue;
        }

        const mapImages = kMapImages[mapName];
        const values = rootValues(overview);

        const verticalSections = buildVerticalSections(values);
        const displayName = typeof values["display_name"] === "string"
            ? values["display_name"]
            : displayNameFromMapName(mapName);
        const mapStyles = buildMapStyles(mapImages);
        const volumes = buildVolumes(values, mapImages, kVolumeBounds[mapName] ?? {});

        if (registry[mapName]) {
            console.warn(`Duplicate map "${mapName}" found in multiple mode folders; ignoring.`);
            continue;
        }

        if (!mapImages.styles["default"]) {
            console.warn(`Map "${mapName}" has no map_default.png; it will have no official radar image.`);
        }

        if (mapImages.styles["default"]?.lower
            && !verticalSections.some((section) => section.name === "lower")) {
            console.warn(`Map "${mapName}" has a map_default_lower.png but no "lower" vertical section.`);
        }

        registry[mapName] = async (): Promise<LoadedMap> => {
            const imageWidth = await probeImageWidth(mapImages.styles["default"]?.default);

            return {
                mapName,
                displayName,

                pos_x: numberOr(values["pos_x"], 0),
                pos_y: numberOr(values["pos_y"], 0),
                scale: numberOr(values["scale"], 1),
                imageWidth,

                verticalSections,
                mapStyles,
                volumes,
            };
        };
    }

    return registry;
};

export const kRegisteredMaps: Record<string, () => Promise<LoadedMap>> = buildMapRegistry();

/* Caches the in-flight/settled promise per map so repeated calls share one result. */
const kMapCache: Record<string, Promise<LoadedMap>> = {};

export const loadMap = async (name: string): Promise<LoadedMap | null> => {
    const mapInfo = kRegisteredMaps[name];
    if (!mapInfo) {
        return null;
    }

    if (!kMapCache[name]) {
        kMapCache[name] = mapInfo();
    }

    return await kMapCache[name];
};
