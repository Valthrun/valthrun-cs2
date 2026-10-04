import { MapDefinition } from "../..";
import Cs2RadarDefault from "./map_style_cs2.png";

export default {
    pos_x: -2308,
    pos_y: 2078,
    scale: 4.082077,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 10000,
            altitudeMin: -10000,
        }
    ],

    mapStyles: [
        {
            name: "Official",
            map: {
                default: Cs2RadarDefault,
            }
        },
    ]
} satisfies MapDefinition;
