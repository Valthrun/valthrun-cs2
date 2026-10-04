import { MapDefinition } from "../..";

export default {
    pos_x: -2087, // upper left world coordinate
    pos_y: 3870,
    scale: 4.9,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 10000,
            altitudeMin: -10000,
        }
    ]
} satisfies MapDefinition;
