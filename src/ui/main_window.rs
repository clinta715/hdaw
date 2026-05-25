use slint::ComponentHandle;

slint::slint! {
    export struct RulerTick {
        position: length,
        label: string,
        is_major: bool,
    }

    export struct ClipInfo {
        id: string,
        track_index: int,
        x: length,
        width: length,
        name: string,
        color: brush,
        fade_in_width: length,
        fade_out_width: length,
        selected: bool,
        track_id: string,
        waveform: image,
    }

    export struct SendInfo {
        id: string,
        target_name: string,
        level: float,
        is_active: bool,
    }

    export struct EffectSlotInfo {
        id: string,
        name: string,
        bypassed: bool,
        selected: bool,
        idx: int,
    }

    export struct ParamInfo {
        name: string,
        value: float,
        min: float,
        max: float,
        display: string,
    }

    export struct TrackInfo {
        id: string,
        index: int,
        label: string,
        volume: float,
        pan: float,
        mute: bool,
        solo: bool,
        armed: bool,
        selected: bool,
        sends: [SendInfo],
        effects: [EffectSlotInfo],
    }

    export struct BusInfo {
        id: string,
        index: int,
        label: string,
        volume: float,
        pan: float,
        mute: bool,
        solo: bool,
        selected: bool,
        sends: [SendInfo],
        effects: [EffectSlotInfo],
    }

    export component MainWindow inherits Window {
        in-out property <bool> can-undo: false;
        in-out property <bool> can-redo: false;
        in-out property <[ClipInfo]> clips: [];
        in-out property <[TrackInfo]> tracks: [];
        in-out property <[BusInfo]> buses: [];
        in-out property <length> pixels-per-second: 50px;
        in-out property <length> playhead-x: 0px;
        in-out property <length> timeline-scroll-x: 0px;
        in-out property <length> timeline-visible-width: 0px;
        in-out property <bool> is-playing: false;
        in-out property <bool> loop-enabled: false;
        in-out property <int> tool-mode: 0;
        in-out property <string> time-display: "0:00.0";
        in-out property <[RulerTick]> ruler-ticks: [];
        in-out property <string> selected-track-id: "";
        in-out property <string> selected-bus-id: "";
        in-out property <int> open-menu: -1;
        in-out property <int> cursor-type: 0;
        in-out property <length> pointer-x: 0px;
        in-out property <length> pointer-y: 0px;
        in-out property <bool> pointer-pressed: false;
        in-out property <string> selected-effect-target: "";
        in-out property <int> selected-effect-index: -1;
        in-out property <bool> selected-effect-is-track: true;
        in-out property <[ParamInfo]> effect-params: [];
        in-out property <string> effect-editor-title: "Effect";
        in-out property <int> fx-menu-target: -1;
        in-out property <int> track-count: 0;
        in-out property <bool> is-recording: false;

        callback undo();
        callback redo();
        callback new-project();
        callback open-project();
        callback save-project();
        callback add-track();
        callback track-selected(string);
        callback track-volume-changed(string, float);
        callback track-pan-changed(string, float);
        callback track-mute-toggled(string);
        callback track-solo-toggled(string);
        callback track-send-level-changed(string, string, float);
        callback bus-selected(string);
        callback bus-volume-changed(string, float);
        callback bus-pan-changed(string, float);
        callback bus-mute-toggled(string);
        callback bus-solo-toggled(string);
        callback bus-send-level-changed(string, string, float);
        callback timeline-pressed(length, length, bool);
        callback timeline-moved(length, length, bool);
        callback timeline-released(length, length);
        callback play();
        callback stop();
        callback toggle-loop();
        callback go-to-start();
        callback go-to-end();
        callback import-file();
        callback quit();

        callback effect-selected(string, int, bool);
        callback add-effect(string, bool, string);
        callback remove-effect(string, bool, int);
        callback toggle-effect-bypass(string, bool, int);
        callback effect-param-changed(string, int, string, float);
        callback start-recording();
        callback track-arm-toggled(string);
        callback delete-track();
        callback delete-bus(string);
        callback move-effect-left(string, bool, int);
        callback move-effect-right(string, bool, int);

        in-out property <string> window-title: "HDAW";

        width: 1280px;
        height: 720px;
        title: root.window-title;

        VerticalLayout {
            padding: 0;

            Rectangle {
                height: 28px;
                background: #252525;

                HorizontalLayout {
                    padding-left: 8px;
                    spacing: 0px;

                    menu-file := Rectangle {
                        min-width: 44px;
                        height: 28px;
                        background: root.open-menu == 0 ? #3a3a3a : transparent;
                        Text {
                            x: 10px;
                            y: (parent.height - 13px) / 2;
                            text: "File";
                            color: root.open-menu == 0 ? #ffffff : #cccccc;
                            font-size: 13px;
                        }
                        TouchArea { clicked => { root.open-menu = root.open-menu == 0 ? -1 : 0; } }
                    }

                    menu-edit := Rectangle {
                        min-width: 40px;
                        height: 28px;
                        background: root.open-menu == 1 ? #3a3a3a : transparent;
                        Text {
                            x: 10px;
                            y: (parent.height - 13px) / 2;
                            text: "Edit";
                            color: root.open-menu == 1 ? #ffffff : #cccccc;
                            font-size: 13px;
                        }
                        TouchArea { clicked => { root.open-menu = root.open-menu == 1 ? -1 : 1; } }
                    }

                    menu-track := Rectangle {
                        min-width: 50px;
                        height: 28px;
                        background: root.open-menu == 3 ? #3a3a3a : transparent;
                        Text {
                            x: 10px;
                            y: (parent.height - 13px) / 2;
                            text: "Track";
                            color: root.open-menu == 3 ? #ffffff : #cccccc;
                            font-size: 13px;
                        }
                        TouchArea { clicked => { root.open-menu = root.open-menu == 3 ? -1 : 3; } }
                    }

                    Rectangle {
                        min-width: 48px;
                        height: 28px;
                        Text {
                            x: 10px;
                            y: (parent.height - 13px) / 2;
                            text: "View";
                            color: #777777;
                            font-size: 13px;
                        }
                    }

                    Rectangle {
                        min-width: 72px;
                        height: 28px;
                        Text {
                            x: 10px;
                            y: (parent.height - 13px) / 2;
                            text: "Transport";
                            color: #777777;
                            font-size: 13px;
                        }
                    }

                    Rectangle {
                        min-width: 40px;
                        height: 28px;
                        Text {
                            x: 10px;
                            y: (parent.height - 13px) / 2;
                            text: "Help";
                            color: #777777;
                            font-size: 13px;
                        }
                    }
                }
            }

            Rectangle {
                height: 32px;
                background: #2a2a2a;

                HorizontalLayout {
                    padding-left: 8px;
                    spacing: 4px;

                    Rectangle {
                        width: 24px;
                        height: 24px;
                        background: root.tool-mode == 0 ? #555555 : #2a2a2a;
                        border-radius: 3px;
                        border-width: 1px;
                        border-color: #444444;
                        Text {
                            text: "◆";
                            color: #cccccc;
                            font-size: 12px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        TouchArea { clicked => { root.tool-mode = 0; } }
                    }

                    Rectangle {
                        width: 24px;
                        height: 24px;
                        background: root.tool-mode == 1 ? #555555 : #2a2a2a;
                        border-radius: 3px;
                        border-width: 1px;
                        border-color: #444444;
                        Text {
                            text: "✂";
                            color: #cccccc;
                            font-size: 12px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        TouchArea { clicked => { root.tool-mode = 1; } }
                    }

                    Rectangle {
                        width: 8px;
                        height: 24px;
                        background: transparent;
                    }

                    Rectangle {
                        width: 24px;
                        height: 24px;
                        background: root.tool-mode == 2 ? #555555 : #2a2a2a;
                        border-radius: 3px;
                        border-width: 1px;
                        border-color: #444444;
                        Text {
                            text: "🧲";
                            color: #cccccc;
                            font-size: 12px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        TouchArea { clicked => { root.tool-mode = 2; } }
                    }

                    Text {
                        text: "  |  ";
                        color: #444444;
                        font-size: 14px;
                        vertical-alignment: center;
                    }

                    Rectangle {
                        width: 24px;
                        height: 24px;
                        background: root.can-undo ? #333333 : #1a1a1a;
                        border-radius: 3px;
                        border-width: 1px;
                        border-color: #444444;
                        Text {
                            text: "↩";
                            color: root.can-undo ? #cccccc : #555555;
                            font-size: 12px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        TouchArea { clicked => { root.undo(); } }
                    }

                    Rectangle {
                        width: 24px;
                        height: 24px;
                        background: root.can-redo ? #333333 : #1a1a1a;
                        border-radius: 3px;
                        border-width: 1px;
                        border-color: #444444;
                        Text {
                            text: "↪";
                            color: root.can-redo ? #cccccc : #555555;
                            font-size: 12px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        TouchArea { clicked => { root.redo(); } }
                    }

                    Text {
                        text: "  |  ";
                        color: #444444;
                        font-size: 14px;
                        vertical-alignment: center;
                    }

                    Rectangle {
                        min-width: 32px;
                        height: 24px;
                        background: transparent;
                        Text {
                            text: "[<<]";
                            color: #888888;
                            font-size: 14px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        TouchArea { clicked => { root.go-to-start(); } }
                    }

                    Rectangle {
                        min-width: 32px;
                        height: 24px;
                        background: transparent;
                        Text {
                            text: "[>>]";
                            color: #888888;
                            font-size: 14px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        TouchArea { clicked => { root.go-to-end(); } }
                    }

                    Rectangle {
                        min-width: 50px;
                        height: 24px;
                        background: transparent;
                        Text {
                            text: root.is-playing ? "▶ Playing" : "▶ Play";
                            color: #4caf50;
                            font-size: 14px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        TouchArea { clicked => { root.play(); } }
                    }

                    Rectangle {
                        min-width: 50px;
                        height: 24px;
                        background: transparent;
                        Text {
                            text: "■ Stop";
                            color: #f44336;
                            font-size: 14px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        TouchArea { clicked => { root.stop(); } }
                    }

                    Rectangle {
                        min-width: 65px;
                        height: 24px;
                        background: transparent;
                        Text {
                            text: root.is-recording ? "● Rec" : "● Rec";
                            color: root.is-recording ? #ff2222 : #884444;
                            font-size: 14px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        TouchArea { clicked => { root.start-recording(); } }
                    }

                    Rectangle {
                        min-width: 50px;
                        height: 24px;
                        background: transparent;
                        Text {
                            text: "[Loop]";
                            color: root.loop-enabled ? #2196f3 : #888888;
                            font-size: 14px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        TouchArea { clicked => { root.toggle-loop(); } }
                    }

                    Text {
                        text: "  |  ";
                        color: #444444;
                        font-size: 14px;
                        vertical-alignment: center;
                    }

                    Rectangle {
                        min-width: 55px;
                        height: 24px;
                        background: #1a3a5c;
                        border-radius: 3px;
                        Text {
                            x: 6px;
                            y: (parent.height - 13px) / 2;
                            text: "Import";
                            color: #64b5f6;
                            font-size: 13px;
                        }
                        TouchArea { clicked => { root.import-file(); } }
                    }
                }
            }

            HorizontalLayout {
                vertical-stretch: 1;

                // TRACK HEADERS (left, 220px)
                Rectangle {
                    width: 220px;
                    background: #222222;
                    clip: true;

                    // Header row matching timeline ruler height
                    Rectangle {
                        y: 0px;
                        width: 100%;
                        height: 24px;
                        background: #1a1a1a;
                        Text {
                            x: 8px;
                            y: (parent.height - 12px) / 2;
                            text: "TRACKS";
                            color: #888888;
                            font-size: 11px;
                        }
                    }

                    for track in root.tracks: Rectangle {
                        y: 24px + track.index * 60px + 4px;
                        width: 216px;
                        height: 52px;
                        background: track.selected ? #3a5a8a : #2a2a2a;
                        border-radius: 3px;
                        x: 2px;

                        Rectangle {
                            x: 2px;
                            y: 0px;
                            width: 4px;
                            height: 100%;
                            background: #4caf50;
                            border-radius: 2px;
                        }

                        Text {
                            x: 10px;
                            y: 2px;
                            text: track.label;
                            color: #cccccc;
                            font-size: 11px;
                            overflow: elide;
                            width: 195px;
                        }

                        Rectangle {
                            x: 10px;
                            y: 16px;
                            width: 195px;
                            height: 10px;
                            background: #1a1a1a;
                            border-radius: 2px;

                            Rectangle {
                                x: 0px;
                                width: parent.width * track.volume;
                                height: 100%;
                                background: #4caf50;
                                border-radius: 2px;
                            }
                        }

                        HorizontalLayout {
                            x: 10px;
                            y: 28px;
                            spacing: 4px;

                            Rectangle {
                                width: 16px;
                                height: 16px;
                                background: track.armed ? #ff2222 : #444444;
                                border-radius: 8px;
                                border-width: 1px;
                                border-color: track.armed ? #ff6666 : #555555;
                                TouchArea { clicked => { root.track-arm-toggled(track.id); } }
                            }
                            Rectangle {
                                width: 22px;
                                height: 16px;
                                background: track.mute ? #f44336 : #555555;
                                border-radius: 2px;
                                Text {
                                    text: "M";
                                    color: #ffffff;
                                    font-size: 10px;
                                    horizontal-alignment: center;
                                    vertical-alignment: center;
                                }
                                TouchArea { clicked => { root.track-mute-toggled(track.id); } }
                            }
                            Rectangle {
                                width: 22px;
                                height: 16px;
                                background: track.solo ? #ffeb3b : #555555;
                                border-radius: 2px;
                                Text {
                                    text: "S";
                                    color: #000000;
                                    font-size: 10px;
                                    horizontal-alignment: center;
                                    vertical-alignment: center;
                                }
                                TouchArea { clicked => { root.track-solo-toggled(track.id); } }
                            }
                        }

                        HorizontalLayout {
                            x: 90px;
                            y: 28px;
                            spacing: 2px;

                            for fx in track.effects: HorizontalLayout {
                                spacing: 0px;

                                Rectangle {
                                    height: 14px;
                                    width: 8px;
                                    background: ta-mvl-t.has-hover ? #3a5a3a : transparent;
                                    border-radius: 2px;
                                    Text {
                                        text: "◀";
                                        color: #668866;
                                        font-size: 6px;
                                        horizontal-alignment: center;
                                        vertical-alignment: center;
                                    }
                                    ta-mvl-t := TouchArea { clicked => { root.move-effect-left(track.id, true, fx.idx); } }
                                }

                                Rectangle {
                                    height: 14px;
                                    min-width: 18px;
                                    background: fx.bypassed ? #333333 : fx.selected ? #3a5a8a : #444444;
                                    border-radius: 2px;
                                    border-width: 1px;
                                    border-color: fx.selected ? #64b5f6 : transparent;
                                    Text {
                                        text: fx.name;
                                        color: fx.bypassed ? #666666 : #cccccc;
                                        font-size: 8px;
                                        horizontal-alignment: center;
                                        vertical-alignment: center;
                                    }
                                    TouchArea {
                                        clicked => { root.effect-selected(track.id, fx.idx, true); }
                                    }
                                }

                                Rectangle {
                                    height: 14px;
                                    width: 8px;
                                    background: ta-mvr-t.has-hover ? #3a5a3a : transparent;
                                    border-radius: 2px;
                                    Text {
                                        text: "▶";
                                        color: #668866;
                                        font-size: 6px;
                                        horizontal-alignment: center;
                                        vertical-alignment: center;
                                    }
                                    ta-mvr-t := TouchArea { clicked => { root.move-effect-right(track.id, true, fx.idx); } }
                                }
                            }

                            Rectangle {
                                height: 14px;
                                width: 18px;
                                background: ta-add-fx-t.has-hover ? #3a5a3a : #333333;
                                border-radius: 2px;
                                Text {
                                    text: "+";
                                    color: #4caf50;
                                    font-size: 8px;
                                    horizontal-alignment: center;
                                    vertical-alignment: center;
                                }
                                ta-add-fx-t := TouchArea { clicked => { root.fx-menu-target = track.index; } }
                            }
                        }

                        TouchArea { clicked => { root.track-selected(track.id); } }
                    }

                    for track in root.tracks: Rectangle {
                        y: 24px + (track.index + 1) * 60px;
                        width: 100%;
                        height: 1px;
                        background: #333333;
                    }

                    Rectangle {
                        y: 24px + root.track-count * 60px + 4px;
                        width: 100%;
                        height: 22px;
                        background: #1a1a1a;
                        Text {
                            x: 8px;
                            y: (parent.height - 12px) / 2;
                            text: "BUSES";
                            color: #888888;
                            font-size: 11px;
                        }
                    }

                    for bus in root.buses: Rectangle {
                        y: 24px + root.track-count * 60px + 28px + bus.index * 60px + 4px;
                        width: 216px;
                        height: 52px;
                        background: bus.selected ? #3a5a8a : #2a2a2a;
                        border-radius: 3px;
                        x: 2px;

                        Rectangle {
                            x: 2px;
                            y: 0px;
                            width: 4px;
                            height: 100%;
                            background: #2196f3;
                            border-radius: 2px;
                        }

                        Text {
                            x: 10px;
                            y: 2px;
                            text: bus.label;
                            color: #cccccc;
                            font-size: 11px;
                            overflow: elide;
                            width: 195px;
                        }

                        Rectangle {
                            x: 10px;
                            y: 16px;
                            width: 195px;
                            height: 10px;
                            background: #1a1a1a;
                            border-radius: 2px;

                            Rectangle {
                                x: 0px;
                                width: parent.width * bus.volume;
                                height: 100%;
                                background: #2196f3;
                                border-radius: 2px;
                            }
                        }

                        HorizontalLayout {
                            x: 10px;
                            y: 28px;
                            spacing: 4px;

                            Rectangle {
                                width: 22px;
                                height: 16px;
                                background: bus.mute ? #f44336 : #555555;
                                border-radius: 2px;
                                Text {
                                    text: "M";
                                    color: #ffffff;
                                    font-size: 10px;
                                    horizontal-alignment: center;
                                    vertical-alignment: center;
                                }
                                TouchArea { clicked => { root.bus-mute-toggled(bus.id); } }
                            }
                            Rectangle {
                                width: 22px;
                                height: 16px;
                                background: bus.solo ? #ffeb3b : #555555;
                                border-radius: 2px;
                                Text {
                                    text: "S";
                                    color: #000000;
                                    font-size: 10px;
                                    horizontal-alignment: center;
                                    vertical-alignment: center;
                                }
                                TouchArea { clicked => { root.bus-solo-toggled(bus.id); } }
                            }
                        }

                        HorizontalLayout {
                            x: 62px;
                            y: 28px;
                            spacing: 2px;

                            for fx in bus.effects: HorizontalLayout {
                                spacing: 0px;

                                Rectangle {
                                    height: 14px;
                                    width: 8px;
                                    background: ta-mvl-b.has-hover ? #3a5a3a : transparent;
                                    border-radius: 2px;
                                    Text {
                                        text: "◀";
                                        color: #668866;
                                        font-size: 6px;
                                        horizontal-alignment: center;
                                        vertical-alignment: center;
                                    }
                                    ta-mvl-b := TouchArea { clicked => { root.move-effect-left(bus.id, false, fx.idx); } }
                                }

                                Rectangle {
                                    height: 14px;
                                    min-width: 18px;
                                    background: fx.bypassed ? #333333 : fx.selected ? #3a5a8a : #444444;
                                    border-radius: 2px;
                                    border-width: 1px;
                                    border-color: fx.selected ? #64b5f6 : transparent;
                                    Text {
                                        text: fx.name;
                                        color: fx.bypassed ? #666666 : #cccccc;
                                        font-size: 8px;
                                        horizontal-alignment: center;
                                        vertical-alignment: center;
                                    }
                                    TouchArea {
                                        clicked => { root.effect-selected(bus.id, fx.idx, false); }
                                    }
                                }

                                Rectangle {
                                    height: 14px;
                                    width: 8px;
                                    background: ta-mvr-b.has-hover ? #3a5a3a : transparent;
                                    border-radius: 2px;
                                    Text {
                                        text: "▶";
                                        color: #668866;
                                        font-size: 6px;
                                        horizontal-alignment: center;
                                        vertical-alignment: center;
                                    }
                                    ta-mvr-b := TouchArea { clicked => { root.move-effect-right(bus.id, false, fx.idx); } }
                                }
                            }

                            Rectangle {
                                height: 14px;
                                width: 18px;
                                background: ta-add-fx-b.has-hover ? #3a5a3a : #333333;
                                border-radius: 2px;
                                Text {
                                    text: "+";
                                    color: #4caf50;
                                    font-size: 8px;
                                    horizontal-alignment: center;
                                    vertical-alignment: center;
                                }
                                ta-add-fx-b := TouchArea { clicked => { root.fx-menu-target = 1000 + bus.index; } }
                            }
                        }

                        Rectangle {
                            x: parent.width - 18px;
                            y: 2px;
                            height: 14px;
                            width: 14px;
                            background: ta-del-b.has-hover ? #553333 : transparent;
                            border-radius: 2px;
                            Text {
                                text: "×";
                                color: #aa6666;
                                font-size: 10px;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                            ta-del-b := TouchArea { clicked => { root.delete-bus(bus.id); } }
                        }

                        TouchArea { clicked => { root.bus-selected(bus.id); } }
                    }

                    for bus in root.buses: Rectangle {
                        y: 24px + root.track-count * 60px + 28px + (bus.index + 1) * 60px;
                        width: 100%;
                        height: 1px;
                        background: #333333;
                    }
                }

                // TIMELINE (fills remaining width)
                Rectangle {
                    horizontal-stretch: 1;
                    background: #1a1a1a;
                    clip: true;

                    Rectangle {
                        y: 0px;
                        width: 100%;
                        height: 24px;
                        background: #222222;

                        Rectangle {
                            y: 23px;
                            width: 100%;
                            height: 1px;
                            background: #444444;
                        }

                        for tick in root.ruler-ticks: Rectangle {
                            x: tick.position - root.timeline-scroll-x;
                            width: 1px;
                            height: tick.is_major ? 12px : 6px;
                            y: tick.is_major ? 0px : 18px;
                            background: #666666;
                        }

                        for tick in root.ruler-ticks: Text {
                            x: tick.position - root.timeline-scroll-x + 3px;
                            y: 12px;
                            text: tick.label;
                            color: #aaaaaa;
                            font-size: 10px;
                            visible: tick.is_major;
                        }

                        Rectangle {
                            x: root.playhead-x - root.timeline-scroll-x - 1px;
                            width: 2px;
                            height: 100%;
                            background: #ff4444;
                        }

                        HorizontalLayout {
                            width: 100%;
                            height: 100%;
                            Rectangle { horizontal-stretch: 1; }
                            Text {
                                text: root.time-display;
                                color: #8bc34a;
                                font-size: 12px;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                            Text {
                                text: "  |  ";
                                color: #444444;
                                font-size: 12px;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                            Text {
                                text: "120.0 BPM";
                                color: #aaaaaa;
                                font-size: 12px;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                            Text {
                                text: "4/4";
                                color: #aaaaaa;
                                font-size: 12px;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                        }
                    }

                    Rectangle {
                        y: 24px;
                        width: 100%;
                        height: parent.height - 24px;
                        clip: true;

                        for clip in root.clips: Rectangle {
                            x: clip.x - root.timeline-scroll-x;
                            width: max(clip.width, 4px);
                            y: clip.track_index * 60px + 4px;
                            height: 52px;
                            background: clip.color;
                            border-radius: 3px;
                            border-width: 1px;
                            border-color: clip.selected ? #ffffff : #333333;

                            Image {
                                x: 0px;
                                y: 14px;
                                width: 100%;
                                height: 34px;
                                source: clip.waveform;
                                image-fit: fill;
                            }

                            Rectangle {
                                x: 0px;
                                width: clip.fade_in_width;
                                height: 20px;
                                background: rgba(0, 0, 0, 0.3);
                            }

                            Rectangle {
                                x: parent.width - clip.fade_out_width;
                                width: clip.fade_out_width;
                                height: 20px;
                                background: rgba(0, 0, 0, 0.3);
                            }

                            Text {
                                x: 4px;
                                y: 2px;
                                text: clip.name;
                                color: #ffffff;
                                font-size: 11px;
                                overflow: elide;
                                width: parent.width - 8px;
                            }

                            Rectangle {
                                x: 0px;
                                width: 5px;
                                height: 100%;
                                background: rgba(255, 255, 255, 0.2);
                            }

                            Rectangle {
                                x: parent.width - 5px;
                                width: 5px;
                                height: 100%;
                                background: rgba(255, 255, 255, 0.2);
                            }

                            Rectangle {
                                x: 0px;
                                y: 0px;
                                width: 10px;
                                height: 10px;
                                background: rgba(255, 255, 255, 0.35);
                            }

                            Rectangle {
                                x: parent.width - 10px;
                                y: 0px;
                                width: 10px;
                                height: 10px;
                                background: rgba(255, 255, 255, 0.35);
                            }
                        }

                        for track in root.tracks: Rectangle {
                            y: (track.index + 1) * 60px;
                            width: 100%;
                            height: 1px;
                            background: #333333;
                        }

                        ta-timeline := TouchArea {
                            width: 100%;
                            height: 100%;
                            mouse-cursor: root.cursor-type == 0 ? MouseCursor.default :
                                          root.cursor-type == 1 ? MouseCursor.w-resize :
                                          root.cursor-type == 2 ? MouseCursor.e-resize :
                                          root.cursor-type == 3 ? MouseCursor.ew-resize :
                                          root.cursor-type == 4 ? MouseCursor.grab :
                                          root.cursor-type == 5 ? MouseCursor.pointer :
                                          root.cursor-type == 6 ? MouseCursor.grabbing : MouseCursor.default;
                            pointer-event(pe) => {
                                root.pointer-x = ta-timeline.mouse-x;
                                root.pointer-y = ta-timeline.mouse-y;
                                root.pointer-pressed = ta-timeline.pressed;
                                if pe.kind == PointerEventKind.down {
                                    root.timeline-pressed(ta-timeline.mouse-x, ta-timeline.mouse-y, false);
                                }
                                if pe.kind == PointerEventKind.up {
                                    root.timeline-released(ta-timeline.mouse-x, ta-timeline.mouse-y);
                                }
                            }
                            moved => { root.timeline-moved(ta-timeline.mouse-x, ta-timeline.mouse-y, false); }
                        }
                    }
                }
            }

            Rectangle {
                height: 120px;
                background: #1e1e1e;
                clip: true;

                HorizontalLayout {
                    padding: 0px;

                    Rectangle {
                        width: 160px;
                        background: #222222;

                        VerticalLayout {
                            padding: 4px;
                            spacing: 4px;

                            Text {
                                text: root.effect-editor-title;
                                color: #cccccc;
                                font-size: 12px;
                            }

                            Rectangle {
                                height: 100%;
                                background: transparent;

                                VerticalLayout {
                                    for param in root.effect-params: Rectangle {
                                        height: 26px;
                                        background: transparent;

                                        HorizontalLayout {
                                            Text {
                                                text: param.name;
                                                color: #aaaaaa;
                                                font-size: 10px;
                                                vertical-alignment: center;
                                                min-width: 60px;
                                            }

                                            Rectangle {
                                                height: 16px;
                                                horizontal-stretch: 1;
                                                background: #1a1a1a;
                                                border-radius: 2px;

                                                Rectangle {
                                                    x: 0px;
                                                    width: parent.width * ((param.value - param.min) / (param.max - param.min));
                                                    height: 100%;
                                                    background: #4caf50;
                                                    border-radius: 2px;
                                                }
                                            }

                                            Text {
                                                text: param.display;
                                                color: #8bc34a;
                                                font-size: 10px;
                                                vertical-alignment: center;
                                                min-width: 40px;
                                                horizontal-alignment: center;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Rectangle {
                        horizontal-stretch: 1;
                        background: #1a1a1a;
                    }
                }
            }
        }

        menu-overlay := Rectangle {
            x: 0px;
            y: 0px;
            width: parent.width;
            height: parent.height;
            clip: false;
            z: 100;

            if root.open-menu == 0: Rectangle {
                x: menu-file.x;
                y: menu-file.y + menu-file.height;
                width: 180px;
                height: 164px;
                background: #2a2a2a;
                border-width: 1px;
                border-color: #444444;
                drop-shadow-blur: 8px;
                drop-shadow-color: #000000;

                VerticalLayout {
                    padding: 4px;
                    spacing: 0px;

                    Rectangle {
                        height: 26px;
                        background: ta-file-new.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout {
                            padding-left: 12px;
                            Text {
                                text: "New Project";
                                color: #cccccc;
                                font-size: 12px;
                                vertical-alignment: center;
                            }
                        }
                        ta-file-new := TouchArea { clicked => { root.open-menu = -1; root.new-project(); } }
                    }
                    Rectangle {
                        height: 26px;
                        background: ta-file-open.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout {
                            padding-left: 12px;
                            Text {
                                text: "Open Project...";
                                color: #cccccc;
                                font-size: 12px;
                                vertical-alignment: center;
                            }
                        }
                        ta-file-open := TouchArea { clicked => { root.open-menu = -1; root.open-project(); } }
                    }
                    Rectangle {
                        height: 26px;
                        background: ta-file-save.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout {
                            padding-left: 12px;
                            Text {
                                text: "Save Project";
                                color: #cccccc;
                                font-size: 12px;
                                vertical-alignment: center;
                            }
                        }
                        ta-file-save := TouchArea { clicked => { root.open-menu = -1; root.save-project(); } }
                    }
                    Rectangle { height: 1px; background: #444444; }
                    Rectangle {
                        height: 26px;
                        background: ta-file-import.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout {
                            padding-left: 12px;
                            Text {
                                text: "Import Audio...";
                                color: #cccccc;
                                font-size: 12px;
                                vertical-alignment: center;
                            }
                        }
                        ta-file-import := TouchArea { clicked => { root.open-menu = -1; root.import-file(); } }
                    }
                    Rectangle { height: 1px; background: #444444; }
                    Rectangle {
                        height: 26px;
                        background: ta-file-quit.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout {
                            padding-left: 12px;
                            Text {
                                text: "Quit";
                                color: #cccccc;
                                font-size: 12px;
                                vertical-alignment: center;
                            }
                        }
                        ta-file-quit := TouchArea { clicked => { root.open-menu = -1; root.quit(); } }
                    }
                }
            }

            if root.open-menu == 1: Rectangle {
                x: menu-edit.x;
                y: menu-edit.y + menu-edit.height;
                width: 160px;
                height: 58px;
                background: #2a2a2a;
                border-width: 1px;
                border-color: #444444;
                drop-shadow-blur: 8px;
                drop-shadow-color: #000000;

                VerticalLayout {
                    padding: 4px;
                    spacing: 0px;

                    Rectangle {
                        height: 26px;
                        background: root.can-undo && ta-edit-undo.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout {
                            padding-left: 12px;
                            Text {
                                text: "Undo";
                                color: root.can-undo ? #cccccc : #666666;
                                font-size: 12px;
                                vertical-alignment: center;
                            }
                        }
                        ta-edit-undo := TouchArea { clicked => { root.open-menu = -1; if root.can-undo { root.undo(); } } }
                    }
                    Rectangle {
                        height: 26px;
                        background: root.can-redo && ta-edit-redo.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout {
                            padding-left: 12px;
                            Text {
                                text: "Redo";
                                color: root.can-redo ? #cccccc : #666666;
                                font-size: 12px;
                                vertical-alignment: center;
                            }
                        }
                        ta-edit-redo := TouchArea { clicked => { root.open-menu = -1; if root.can-redo { root.redo(); } } }
                    }
                }
            }

            if root.open-menu == 3: Rectangle {
                x: menu-track.x;
                y: menu-track.y + menu-track.height;
                width: 160px;
                height: 62px;
                background: #2a2a2a;
                border-width: 1px;
                border-color: #444444;
                drop-shadow-blur: 8px;
                drop-shadow-color: #000000;

                VerticalLayout {
                    padding: 4px;
                    spacing: 0px;

                    Rectangle {
                        height: 26px;
                        background: ta-track-add.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout {
                            padding-left: 12px;
                            Text {
                                text: "Add Track";
                                color: #cccccc;
                                font-size: 12px;
                                vertical-alignment: center;
                            }
                        }
                        ta-track-add := TouchArea { clicked => { root.open-menu = -1; root.add-track(); } }
                    }
                    Rectangle { height: 1px; background: #444444; }
                    Rectangle {
                        height: 26px;
                        background: ta-track-del.has-hover ? #553333 : transparent;
                        HorizontalLayout {
                            padding-left: 12px;
                            Text {
                                text: "Delete Track";
                                color: #cc8888;
                                font-size: 12px;
                                vertical-alignment: center;
                            }
                        }
                        ta-track-del := TouchArea { clicked => { root.open-menu = -1; root.delete-track(); } }
                    }
                }
            }

            if root.fx-menu-target >= 0: Rectangle {
                x: 0px;
                y: 0px;
                width: 100%;
                height: 100%;
                background: rgba(0, 0, 0, 0.5);

                Rectangle {
                    x: (parent.width - 160px) / 2;
                    y: (parent.height - 160px) / 2;
                    width: 160px;
                    height: 160px;
                    background: #2a2a2a;
                    border-width: 1px;
                    border-color: #444444;
                    drop-shadow-blur: 8px;
                    drop-shadow-color: #000000;

                    VerticalLayout {
                        padding: 4px;
                        spacing: 0px;

                        Text {
                            text: "Add Effect";
                            color: #ffffff;
                            font-size: 13px;
                            horizontal-alignment: center;
                            padding: 4px;
                        }

                        Rectangle { height: 1px; background: #444444; }

                        Rectangle {
                            height: 28px;
                            background: ta-add-eq.has-hover ? #3a5a8a : transparent;
                            HorizontalLayout {
                                padding-left: 12px;
                                Text {
                                    text: "Equalizer";
                                    color: #cccccc;
                                    font-size: 12px;
                                    vertical-alignment: center;
                                }
                            }
                            ta-add-eq := TouchArea {
                                clicked => {
                                    if root.fx-menu-target < 1000 {
                                        root.add-effect(root.tracks[root.fx-menu-target].id, true, "Equalizer");
                                    } else {
                                        root.add-effect(root.buses[root.fx-menu-target - 1000].id, false, "Equalizer");
                                    }
                                    root.fx-menu-target = -1;
                                }
                            }
                        }

                        Rectangle {
                            height: 28px;
                            background: ta-add-comp.has-hover ? #3a5a8a : transparent;
                            HorizontalLayout {
                                padding-left: 12px;
                                Text {
                                    text: "Compressor";
                                    color: #cccccc;
                                    font-size: 12px;
                                    vertical-alignment: center;
                                }
                            }
                            ta-add-comp := TouchArea {
                                clicked => {
                                    if root.fx-menu-target < 1000 {
                                        root.add-effect(root.tracks[root.fx-menu-target].id, true, "Compressor");
                                    } else {
                                        root.add-effect(root.buses[root.fx-menu-target - 1000].id, false, "Compressor");
                                    }
                                    root.fx-menu-target = -1;
                                }
                            }
                        }

                        Rectangle {
                            height: 28px;
                            background: ta-add-rev.has-hover ? #3a5a8a : transparent;
                            HorizontalLayout {
                                padding-left: 12px;
                                Text {
                                    text: "Reverb";
                                    color: #cccccc;
                                    font-size: 12px;
                                    vertical-alignment: center;
                                }
                            }
                            ta-add-rev := TouchArea {
                                clicked => {
                                    if root.fx-menu-target < 1000 {
                                        root.add-effect(root.tracks[root.fx-menu-target].id, true, "Reverb");
                                    } else {
                                        root.add-effect(root.buses[root.fx-menu-target - 1000].id, false, "Reverb");
                                    }
                                    root.fx-menu-target = -1;
                                }
                            }
                        }

                        Rectangle {
                            height: 28px;
                            background: ta-add-del.has-hover ? #3a5a8a : transparent;
                            HorizontalLayout {
                                padding-left: 12px;
                                Text {
                                    text: "Delay";
                                    color: #cccccc;
                                    font-size: 12px;
                                    vertical-alignment: center;
                                }
                            }
                            ta-add-del := TouchArea {
                                clicked => {
                                    if root.fx-menu-target < 1000 {
                                        root.add-effect(root.tracks[root.fx-menu-target].id, true, "Delay");
                                    } else {
                                        root.add-effect(root.buses[root.fx-menu-target - 1000].id, false, "Delay");
                                    }
                                    root.fx-menu-target = -1;
                                }
                            }
                        }

                        Rectangle {
                            height: 28px;
                            background: ta-cancel.has-hover ? #553333 : transparent;
                            HorizontalLayout {
                                padding-left: 12px;
                                Text {
                                    text: "Cancel";
                                    color: #aa6666;
                                    font-size: 12px;
                                    vertical-alignment: center;
                                }
                            }
                            ta-cancel := TouchArea { clicked => { root.fx-menu-target = -1; } }
                        }
                    }
                }
            }
        }
    }
}

pub fn run() {
    let window = MainWindow::new().unwrap();
    window.run().unwrap();
}
