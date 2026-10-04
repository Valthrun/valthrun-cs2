import { MapDefinition } from "../..";

export default {
    pos_x: -1316, // upper left world coordinate
    pos_y: 1288,
    scale: 2.539062,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 10000,
            altitudeMin: -5,
        },
        {
            name: "lower",
            altitudeMax: -5,
            altitudeMin: -10000,
        }
    ]
} satisfies MapDefinition;
