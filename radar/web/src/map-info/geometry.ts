import { F32 } from "../backend/definitions";
import { LoadedMap, MapVolume } from "./index";

export type MapPosition = { left: number, top: number };

/*
 * A map's radar levels. Most maps only have vertical levels (default / lower).
 * Maps like rush_001 are made of many separate rooms, each with its own radar
 * image and coordinate transform, so a level can also be a named volume.
 */

/* Rush only has one room active per round, so remember it to avoid scanning. */
let currentVolumeName: string | null = null;

export const resetMapVolumeCache = (): void => {
    currentVolumeName = null;
};

const findVolume = (map: LoadedMap, position: [number, number, number]): MapVolume | null => {
    if (map.volumes.length === 0) {
        return null;
    }

    /* One room is active at a time, so the last one is the best first guess. */
    if (currentVolumeName) {
        const previous = map.volumes.find(volume => volume.name === currentVolumeName);
        if (previous && containsVolume(previous, position)) {
            return previous;
        }
    }

    for (const volume of map.volumes) {
        if (containsVolume(volume, position)) {
            currentVolumeName = volume.name;
            return volume;
        }
    }

    currentVolumeName = null;
    return null;
};

const containsVolume = (volume: MapVolume, position: [number, number, number]): boolean => {
    return position[0] >= volume.worldMinX && position[0] < volume.worldMaxX
        && position[1] >= volume.worldMinY && position[1] < volume.worldMaxY;
};

export const getMapLevel = (map: LoadedMap, position: [F32, F32, F32]): string => {
    const volume = findVolume(map, position);
    if (volume) {
        return volume.name;
    }

    return map.verticalSections.find(section => section.altitudeMin <= position[2] && position[2] < section.altitudeMax)?.name ?? "default";
};

export const getMapPosition = (map: LoadedMap, position: [number, number, number]): [number, number] => {
    const volume = map.volumes.length > 0 ? findVolume(map, position) : null;
    const mapSize = (volume?.scale ?? map.scale) * 1024;

    return [
        (position[0] - (volume?.pos_x ?? map.pos_x)) * 100 / mapSize,
        (position[1] - (volume?.pos_y ?? map.pos_y)) * 100 / -mapSize
    ];
};

/** The image for a level, which is a volume image on rush maps and a style otherwise. */
export const getMapLevelImage = (map: LoadedMap, styleName: string, level: string): string | undefined => {
    const volume = map.volumes.find(volume => volume.name === level);
    if (volume) {
        return level.endsWith("_lower") ? volume.lowerImage : (level === volume.name ? volume.image : undefined);
    }

    const style = map.mapStyles.find(style => style.name === styleName) ?? map.mapStyles[0];
    return style?.map[level as keyof typeof style.map];
};
