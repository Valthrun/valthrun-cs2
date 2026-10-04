import { MapDefinition } from "../..";

export default {
    pos_x: -3230, // upper left world coordinate
    pos_y: 1713,
    scale: 5,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 10000,
            altitudeMin: -10000,
        }
    ]
} satisfies MapDefinition;
