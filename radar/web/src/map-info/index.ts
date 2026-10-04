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

export type MapDefinition = Omit<LoadedMap, "mapName" | "displayName">;

const kMapContext = import.meta.webpackContext(".", {
    recursive: true,
    regExp: /^\.\/[^/]+\/[^/]+\/index\.ts$/,
    mode: "lazy",
    chunkName: "map-[request]",
});

const displayNameFromMapName = (mapName: string): string => {
    const suffix = mapName.includes("_") ? mapName.slice(mapName.indexOf("_") + 1) : mapName;
    return suffix
        .split("_")
        .filter((part) => part.length > 0)
        .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
        .join(" ");
};

const buildMapRegistry = (): Record<string, () => Promise<LoadedMap>> => {
    const registry: Record<string, () => Promise<LoadedMap>> = {};

    for (const request of kMapContext.keys()) {
        const segments = request.split("/");
        const mapName = segments[segments.length - 2];
        const displayName = displayNameFromMapName(mapName);

        /*
        if (registry[mapName]) {
            console.warn(`Duplicate map "${mapName}" found in multiple mode folders; ignoring "${request}".`);
            continue;
        }
        */

        registry[mapName] = () =>
            kMapContext(request).then((value: { default?: MapDefinition }): LoadedMap => {
                if (!value.default) {
                    throw new Error(`Map "${mapName}" does not have a default export.`);
                }

                return {
                    ...value.default,
                    mapName,
                    displayName,
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
