import { Box, Typography } from "@mui/material";
import * as React from "react";
import { kDefaultRadarState, UpdateStatistics } from "../../../../backend/connection";
import { LoadedMap, loadMap } from "../../../../map-info";
import { getMapLevel, getMapPosition, getMapLevelImage, resetMapVolumeCache } from "../../../../map-info/geometry";
import ImageBomb from "../../../../assets/bomb.png";
import { useAppSelector } from "../../../../state";
import BombIndicator from "../../../components/bomb/bomb-indicator";
import IconPlayerDead from "./icon_player_dead.svg";
import IconPlayer from "./icon_player.svg";
import { F32, RadarState } from "../../../../backend/definitions";
import SizedContainer from "../../../components/container/sized-container";
import SquareContainer, { useSquareSize } from "../../../components/container/square-container";
import { useQuery } from "react-query";
import { useSubscriberClient } from "../../../components/connection";
import { shallowEqual } from "../../../utils/compare";

const kUninitialized = Symbol("uninitialized");
const useRadarState = <V extends any>(selector: (state: RadarState) => V): V => {
    const internalState = React.useRef<{ value: V | typeof kUninitialized, selector: any }>({ value: kUninitialized, selector });
    const [_versionId, setVersionId] = React.useState(0);
    const subscriber = useSubscriberClient();

    if (internalState.current.selector !== selector || internalState.current.value === kUninitialized) {
        /* state needs recalculation */
        internalState.current = {
            selector,
            value: selector(subscriber.getCurrentRadarState() ?? kDefaultRadarState)
        }
    }

    React.useEffect(() => {
        return subscriber.events.on("radar.state", state => {
            const newValue = selector(state);
            if (shallowEqual(internalState.current.value, newValue)) {
                /* value not changed */
                return;
            }

            internalState.current.value = newValue;
            setVersionId(versionId => versionId + 1);
        });
    }, [selector, subscriber]);

    return internalState.current.value as V;
}

const useQueryMap = (worldName: string) => {
    return useQuery({
        queryKey: ["map-info", worldName],
        queryFn: async () => await loadMap(worldName),
        enabled: !worldName.includes("empty")
    });
};

const useCurrentMap = () => {
    const worldName = useRadarState(
        React.useCallback(state => state.worldName, [])
    );
    return useQueryMap(worldName).data ?? null;
}

const MapTitle = React.memo(() => {
    const worldName = useRadarState(
        React.useCallback(state => state.worldName, [])
    );
    const queryMap = useQueryMap(worldName);

    const hideMapTitle = useAppSelector(state => state.radarSettings.hideMapTitle);
    if (hideMapTitle) {
        return null;
    }

    return (
        <Typography variant={"h5"}>{queryMap.data?.displayName ?? worldName}</Typography>
    );
});

export const RadarRenderer = React.memo(() => {
    const client = useSubscriberClient();
    const worldName = useRadarState(React.useCallback(state => state.worldName, []));
    const queryMap = useQueryMap(worldName);

    const [displayBombDetails, displayStatistics] = useAppSelector(state => [
        state.radarSettings.displayBombDetails,
        state.radarSettings.displayStatistics,
    ], shallowEqual);
    const padding = useAppSelector(state => state.radarSettings.disablePadding ? 0 : 3);

    const renderStatistics = React.useMemo(() => new UpdateStatistics(), []);
    const isInMatch = !worldName.includes("empty");
    return (
        <Box
            sx={{
                height: "100%",
                width: "100%",

                display: "flex",
                flexDirection: "column",

                p: padding,
            }}
        >
            <MapTitle />
            <Box
                sx={{
                    height: "100%",
                    width: "100%",

                    display: "flex",
                    flexDirection: "row",

                    position: "relative",
                    p: 3,
                }}
            >
                <Box
                    sx={{
                        position: "absolute",
                        zIndex: 1,

                        top: "1em",
                        left: 0,
                        right: 0,

                        display: "flex",
                        flexDirection: "row",
                        justifyContent: "center",
                    }}
                >
                    {displayBombDetails && <RenderBombIndicator />}
                </Box>

                {isInMatch && queryMap.isSuccess && (
                    queryMap.data ? (
                        <MapContainer renderStatistics={renderStatistics} />
                    ) : (
                        <Box sx={{ position: "absolute", top: 0, left: 0, right: 0, bottom: 0, display: "flex", flexDirection: "column", justifyContent: "center" }}>
                            <Typography variant={"h5"} sx={{ alignSelf: "center", color: "error.dark" }}>
                                Map Unknown
                            </Typography>
                        </Box>
                    )
                )}
                {isInMatch && (queryMap.isLoading || queryMap.isError) && (
                    <Box sx={{ position: "absolute", top: 0, left: 0, right: 0, bottom: 0, display: "flex", flexDirection: "column", justifyContent: "center" }}>
                        {queryMap.isLoading ? (
                            <Typography variant={"h5"} sx={{ alignSelf: "center", color: "grey.500" }}>
                                loading map info
                            </Typography>
                        ) : (
                            <Typography variant={"h5"} sx={{ alignSelf: "center", color: "palette.error.dark" }}>
                                <React.Fragment>
                                    Failed to load map.<br />
                                    Lookup the console for more details.
                                </React.Fragment>
                            </Typography>
                        )}
                    </Box>
                )}
                {!isInMatch && (
                    <Box sx={{ position: "absolute", top: 0, left: 0, right: 0, bottom: 0, display: "flex", flexDirection: "column", justifyContent: "center" }}>
                        <Typography variant={"h5"} sx={{ alignSelf: "center", color: "grey.500" }}>
                            waiting for match
                        </Typography>
                    </Box>
                )}
                {displayStatistics && (
                    <Box sx={{
                        position: "absolute",
                        top: 0,
                        left: 0,

                        display: "flex",
                        flexDirection: "column",

                        zIndex: 1,
                        backgroundColor: "rgba(0, 0, 0, .6)",
                        p: 2,

                        minHeight: "1em",
                        width: "18em",
                    }}>
                        <Typography>Updates:</Typography>
                        <DisplayStatistics statistics={client.stateUpdateStatistics} />

                        <Typography sx={{ mt: 1 }}>Renderer:</Typography>
                        <DisplayStatistics statistics={renderStatistics} />
                    </Box>
                )}
            </Box>
        </Box>
    );
});

const RenderBombIndicator = React.memo(() => {
    const plantedC4 = useRadarState(React.useCallback(state => state.plantedC4, []));
    if (!plantedC4) {
        return null;
    }

    return (
        <BombIndicator state={plantedC4.state} />
    );
});

const MapContainer = React.memo((props: { renderStatistics: UpdateStatistics }) => {
    const refContainer = React.useRef(null);

    const currentMap = useCurrentMap();
    const [showAllLayers, marginLeft, marginRight, marginTop, marginBottom] = useAppSelector(state => [
        state.radarSettings.showAllLayers,
        state.radarSettings.mapMarginLeft,
        state.radarSettings.mapMarginRight,
        state.radarSettings.mapMarginTop,
        state.radarSettings.mapMarginBottom,
    ], shallowEqual);

    const localMapLevel = useRadarState(React.useCallback(state => {
        const position = state.playerPawns.find(pawn => pawn.controllerEntityId === state.localControllerEntityId)?.position ?? [0, 0, 0];
        return getMapLevel(currentMap, position);
    }, [currentMap]));

    React.useEffect(() => resetMapVolumeCache(), [currentMap]);

    return (
        <React.Fragment>
            <CssVariableProvider targetRef={refContainer} renderStatistics={props.renderStatistics} />
            <SizedContainer ref={refContainer} sx={{
                display: "flex",
                flexDirection: "column",
                justifyContent: "center",

                left: `${marginLeft}%`,
                right: `${marginRight}%`,
                top: `${marginTop}%`,
                bottom: `${marginBottom}%`,
            }}>
                {size => {
                    const isVolume = currentMap.volumes.some(volume => volume.name === localMapLevel);
                    if (showAllLayers && currentMap.verticalSections.length > 1 && !isVolume) {
                        const minAxis = Math.min(size.width, size.height);
                        const maxAxis = Math.max(size.width, size.height);

                        const squareSize = Math.min(minAxis, maxAxis / 2);
                        return (
                            <Box sx={{
                                display: "flex",
                                flexDirection: "row",
                                flexWrap: "wrap",

                                justifyContent: "center",
                                alignSelf: "center",
                            }}>
                                {currentMap.verticalSections.map(section => (
                                    <SquareContainer squareSize={squareSize} key={section.name}>
                                        <MapLevel level={section.name} />
                                    </SquareContainer>
                                ))}
                            </Box>
                        )
                    } else {
                        const minAxis = Math.min(size.width, size.height);
                        return (
                            <SquareContainer
                                squareSize={minAxis}
                                sx={{
                                    alignSelf: "center",
                                }}
                            >
                                <MapLevel level={localMapLevel} />
                            </SquareContainer>
                        );
                    }

                }}
            </SizedContainer>
        </React.Fragment>
    );
});

const MapImage = React.memo((props: { level: string }) => {
    const currentMap = useCurrentMap();
    const mapStyle = useAppSelector(state => state.radarSettings.mapStyle);
    const image = React.useMemo(() => getMapLevelImage(currentMap, mapStyle, props.level), [currentMap, mapStyle, props.level]);
    return (
        <Box
            sx={{
                height: "100%",
                width: "100%",
                backgroundImage: `url("${image ?? ""}")`,
                backgroundPosition: "center",
                backgroundSize: "cover",
            }}
        />
    );
});

const CssVariableProvider = (props: { targetRef: React.RefObject<HTMLElement>, renderStatistics: UpdateStatistics }): null => {
    const { targetRef, renderStatistics } = props;

    const currentMap = useCurrentMap();
    const subscriber = useSubscriberClient();

    React.useEffect(() => {
        const target = targetRef.current;
        if (!target) {
            return;
        }

        let currentRequestFrame: number | null = null;
        const executeFrame = () => {
            /* clear the last request */
            currentRequestFrame = null;

            const state = subscriber.getCurrentRadarState();
            if (!state) {
                /* nothing to update */
                return;
            }

            renderStatistics.logUpdate();

            const variables: string[] = [];

            for (const { pawnEntityId, position, rotation } of state.playerPawns) {
                const mapPosition = getMapPosition(currentMap, position);
                variables.push(`--pawn-${pawnEntityId}-left: ${mapPosition[0]}`);
                variables.push(`--pawn-${pawnEntityId}-top: ${mapPosition[1]}`);
                variables.push(`--pawn-${pawnEntityId}-rotate: ${-rotation + 90}deg`);
            }

            for (const { entityId, position } of state.c4Entities) {
                const mapPosition = getMapPosition(currentMap, position);
                variables.push(`--c4-${entityId}-left: ${mapPosition[0]}`);
                variables.push(`--c4-${entityId}-top: ${mapPosition[1]}`);
            }

            const localPlayer = state.playerPawns.find(pawn => pawn.controllerEntityId === state.localControllerEntityId);
            if (localPlayer) {
                const mapPosition = getMapPosition(currentMap, localPlayer.position);
                variables.push(`--map-transform-origin-left: ${mapPosition[0] / 100}`);
                variables.push(`--map-transform-origin-top: ${mapPosition[1] / 100}`);
            }

            variables.push(`--update-interval: 50ms`);
            target.style = variables.join(";\n");
        };

        const unsubscrbe = subscriber.events.on("radar.state", () => {
            if (!currentRequestFrame) {
                currentRequestFrame = requestAnimationFrame(executeFrame);
            }
        });

        return () => {
            unsubscrbe();
            if (currentRequestFrame) {
                cancelAnimationFrame(currentRequestFrame);
                currentRequestFrame = null;
            }
        };
    }, [targetRef, renderStatistics, subscriber, currentMap]);
    return null;
};

const MapLevel = React.memo((props: { level: string }) => {
    const { level } = props;
    const currentMap = useCurrentMap();

    const visiblePawnIds = useRadarState(React.useCallback(state => state.playerPawns.filter(pawn => getMapLevel(currentMap, pawn.position) === level).map(pawn => pawn.pawnEntityId), [currentMap, level]));
    const visibleC4EntityIds = useRadarState(React.useCallback(state => state.c4Entities.filter(entity => getMapLevel(currentMap, entity.position) === level).map(entity => entity.entityId), [currentMap, level]));
    const plantedC4Position = useRadarState(React.useCallback(state => state.plantedC4 && getMapLevel(currentMap, state.plantedC4.position) === level ? state.plantedC4.position : null, [currentMap, level]));

    const [mapScale, colorDotCT, colorDotT, colorDotOwn] = useAppSelector(state => [
        state.radarSettings.mapScale,
        state.radarSettings.colorDotCT,
        state.radarSettings.colorDotT,
        state.radarSettings.colorDotOwn
    ], shallowEqual);

    return (
        <Box
            sx={{
                position: "relative",

                transformOrigin: "calc(var(--map-transform-origin-left) * var(--square-size)) calc(var(--map-transform-origin-top) * var(--square-size))",
                transform: `scale(${mapScale})`,
                transition: "all var(--update-interval) linear",

                height: "100%",
                width: "100%",

                ".icon_player_svg__view-cone": {
                    fill: "#fff",
                },
                ".team-t": {
                    ".icon_player_svg__player-dot, .icon_player_dead_svg__player_cross": {
                        fill: colorDotT,
                    },
                },
                ".team-ct": {
                    ".icon_player_svg__player-dot, .icon_player_dead_svg__player_cross": {
                        fill: colorDotCT,
                    },
                },
                ".broadcaster": {
                    ".icon_player_svg__player-dot, .icon_player_dead_svg__player_cross": {
                        fill: colorDotOwn,
                    },
                },

                "svg": {
                    transition: "all var(--update-interval) linear"
                }
            }}
        >
            <MapImage level={props.level} />
            {visiblePawnIds.map(pawnId => <MapPlayerPawn key={`pawn-${pawnId}`} pawnId={pawnId} />)}
            {visibleC4EntityIds.map(entityId => <MapC4 key={`c4-${entityId}`} entityId={entityId} />)}
            {plantedC4Position ? <MapIconC4 position={getMapPosition(currentMap, plantedC4Position)} key="planted-c4" /> : null}
        </Box>
    );
});


export const MapPlayerPawn = React.memo((props: { pawnId: number }) => {
    const highlightOwn = useAppSelector((state) => state.radarSettings.showDotOwn);
    const iconProps = useRadarState(React.useCallback(state => {
        const pawn = state.playerPawns.find(pawn => pawn.pawnEntityId === props.pawnId);
        if (!pawn) {
            return null;
        }

        return {
            pawnId: pawn.pawnEntityId,
            team: pawn.teamId === 3 ? "ct" : "t",
            status: pawn.playerHealth > 0 ? "alive" : "dead",
            isBroadcaster: highlightOwn && pawn.controllerEntityId === state.localControllerEntityId
        } satisfies MapIconPawnProps;
    }, [highlightOwn, props.pawnId]));

    if (!iconProps) {
        /* invalid pawn id */
        return null;
    }

    return (
        <MapIconPawn {...iconProps} />
    );
});

type MapIconPawnProps = {
    pawnId: number,

    team: "t" | "ct",
    status: "alive" | "dead",

    isBroadcaster: boolean;
};
export const MapIconPawn = (props: MapIconPawnProps) => {
    const { pawnId, isBroadcaster, team, status } = props;
    const mapWidth = useSquareSize();
    const iconSize = useAppSelector((state) => state.radarSettings.iconSize);
    const iconWidth = (mapWidth * iconSize) / 100;

    let Icon;
    if (status === "dead") {
        Icon = IconPlayerDead;
    } else {
        Icon = IconPlayer;
    }

    return (
        <Icon
            style={{
                position: "absolute",

                top: `calc(var(--pawn-${pawnId}-top) * ${mapWidth / 100}px - ${iconWidth / 2}px)`,
                left: `calc(var(--pawn-${pawnId}-left) * ${mapWidth / 100}px - ${iconWidth / 2}px)`,
                rotate: `var(--pawn-${pawnId}-rotate)`,

                filter: "drop-shadow(-2px -2px 3px rgba(0, 0, 0, .5))",
            }}
            width={iconWidth}
            className={`animated team-${team} ${isBroadcaster ? "broadcaster" : ""}`}
        />
    );
};

const MapC4 = React.memo((props: { entityId: number }) => {
    const { entityId } = props;

    const mapWidth = useSquareSize();
    const iconSize = useAppSelector((state) => state.radarSettings.iconSize);
    const iconWidth = (mapWidth * iconSize) / 100;

    return (
        <Box
            sx={{
                top: `calc(var(--c4-${entityId}-top) * ${mapWidth / 100}px - ${iconWidth / 2}px)`,
                left: `calc(var(--c4-${entityId}-left) * ${mapWidth / 100}px - ${iconWidth / 2}px)`,

                height: `${iconWidth}px`,
                width: `${iconWidth}px`,

                position: "absolute",

                backgroundImage: `url("${ImageBomb}")`,
                backgroundPosition: "center",
                backgroundSize: "contain",
            }}
        />
    );
});

const MapIconC4 = (props: { position: [number, number] }) => {
    const [bombX, bombY] = props.position;
    const iconSize = useAppSelector((state) => state.radarSettings.iconSize);
    return (
        <Box
            sx={{
                top: "var(--pos-y)",
                left: "var(--pos-x)",

                height: `${iconSize}%`,
                width: `${iconSize}%`,

                position: "absolute",

                backgroundImage: `url("${ImageBomb}")`,
                backgroundPosition: "center",
                backgroundSize: "contain",
            }}
            style={
                {
                    "--pos-x": `${bombX - iconSize / 2}%`,
                    "--pos-y": `${bombY - iconSize / 2}%`,
                } as any
            }
        />
    );
};

const DisplayStatistics = React.memo((props: { statistics: UpdateStatistics }) => {
    const { statistics } = props;
    const [_renderId, setRenderId] = React.useState(0);

    React.useEffect(() => {
        const updateId = setInterval(() => setRenderId(value => value + 1), 250);
        return () => clearInterval(updateId);
    }, []);


    const history = statistics.getHistory();
    const average = statistics.getAverageInterval();
    const maxTime = history.reduce((max, current) => Math.max(max, current), history[0]);
    return (
        <React.Fragment>
            <Typography>Update time: {average.toFixed(0)}ms ({Math.round(1000 / average)} ups)</Typography>
            <Typography>Max time: {maxTime.toFixed(0)}ms</Typography>
        </React.Fragment>
    )
});