/* Finding Maps Source
Find Counter-Strike Global Offensive\game\csgo\pak01_dir.vpk
Open with Source Viewer ( https://github.com/ValveResourceFormat/ValveResourceFormat )
Locate panorama/images/overheadmaps
* */
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

export type LoadedMap = {
    mapName: string;
    displayName: string;

    pos_x: number,
    pos_y: number,
    scale: number,

    verticalSections: VerticalSection[],
    mapStyles: MapStyle[]
};

export type MapDefinition = Omit<LoadedMap, "mapName" | "displayName" | "mapStyles">;

const kMapContext = import.meta.webpackContext(".", {
    recursive: true,
    regExp: /^\.\/[^/]+\/[^/]+\/index\.ts$/,
    mode: "lazy",
    chunkName: "map-[request]",
});

const kMapImageContext = import.meta.webpackContext(".", {
    recursive: true,
    regExp: /^\.\/[^/]+\/[^/]+\/map_(default|simple)(_lower)?\.png$/,
    mode: "sync",
});

/** Turns a map folder name such as "de_dust2" into a display name such as "Dust2". */
const displayNameFromMapName = (mapName: string): string => {
    const suffix = mapName.includes("_") ? mapName.slice(mapName.indexOf("_") + 1) : mapName;
    return suffix
        .split("_")
        .filter((part) => part.length > 0)
        .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
        .join(" ");
};

/** Style name order, which also defines the fallback order (Official first). */
const kStyleOrder = ["Official", "SimpleRadar"] as const;

type StyleImage = { default: string, lower?: string };

const buildMapImages = (): Record<string, Record<string, StyleImage>> => {
    const images: Record<string, Record<string, StyleImage>> = {};

    for (const request of kMapImageContext.keys()) {
        /* request looks like "./competitive/de_dust2/map_default_lower.png" */
        const segments = request.split("/");
        const mapName = segments[segments.length - 2];
        const fileName = segments[segments.length - 1];

        const match = /^map_(default|simple)(_lower)?\.png$/.exec(fileName);
        if (!match) {
            continue;
        }

        const styleName = match[1] === "default" ? "Official" : "SimpleRadar";
        const isLower = Boolean(match[2]);

        if (!images[mapName]) {
            images[mapName] = {};
        }
        if (!images[mapName][styleName]) {
            images[mapName][styleName] = { default: "" };
        }

        const imageModule = kMapImageContext(request);
        const url: string = imageModule && imageModule.default ? imageModule.default : imageModule;

        if (isLower) {
            images[mapName][styleName].lower = url;
        } else {
            images[mapName][styleName].default = url;
        }
    }

    return images;
};

const kMapImages = buildMapImages();

const buildMapStyles = (mapImages: Record<string, StyleImage> | undefined): MapStyle[] => {
    const styles: MapStyle[] = [];
    if (!mapImages) {
        return styles;
    }

    for (const styleName of kStyleOrder) {
        const image = mapImages[styleName];
        if (!image || !image.default) {
            continue;
        }

        const map: MapStyle["map"] = { default: image.default };
        if (image.lower) {
            map.lower = image.lower;
        }

        styles.push({ name: styleName, map });
    }

    return styles;
};

const buildMapRegistry = (): Record<string, () => Promise<LoadedMap>> => {
    const registry: Record<string, () => Promise<LoadedMap>> = {};

    for (const request of kMapContext.keys()) {
        const segments = request.split("/");
        const mapName = segments[segments.length - 2];
        const displayName = displayNameFromMapName(mapName);
        const mapImages = kMapImages[mapName];
        const mapStyles = buildMapStyles(mapImages);

        /*
        if (registry[mapName]) {
            console.warn(`Duplicate map "${mapName}" found in multiple mode folders; ignoring "${request}".`);
            continue;
        }

        if (!mapImages || !mapImages["Official"]) {
            console.warn(`Map "${mapName}" has no map_default.png; it will have no official radar image.`);
        }
        */

        registry[mapName] = () =>
            kMapContext(request).then((value: { default?: MapDefinition }): LoadedMap => {
                if (!value.default) {
                    throw new Error(`Map "${mapName}" does not have a default export.`);
                }

                if (mapImages && mapImages["Official"] && mapImages["Official"].lower
                    && !value.default.verticalSections.some((section) => section.name === "lower")) {
                    console.warn(`Map "${mapName}" has a map_default_lower.png but no "lower" vertical section.`);
                }

                return {
                    ...value.default,
                    mapName,
                    displayName,
                    mapStyles,
                };
            });
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
