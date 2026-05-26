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
        auto_image: image,
        auto_param_name: string,
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

    export struct PoolEntry {
        name: string,
        info: string,
        usage: int,
        path: string,
        idx: int,
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
        input_monitoring: bool,
        selected: bool,
        track_color: brush,
        sends: [SendInfo],
        effects: [EffectSlotInfo],
        peak_l: float,
        peak_r: float,
        output_name: string,
        auto_param_name: string,
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
        track_color: brush,
        sends: [SendInfo],
        effects: [EffectSlotInfo],
        peak_l: float,
        peak_r: float,
        output_name: string,
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
        in-out property <bool> snap-enabled: false;
        in-out property <int> snap-mode: 4;
        in-out property <int> snap-param: 2;
        in-out property <bool> snap-menu-open: false;
        in-out property <bool> pool-visible: false;
        in-out property <[PoolEntry]> pool-entries: [];
        in-out property <string> bpm-display: "120.0 BPM";
        in-out property <string> time-sig-display: "4/4";
        in-out property <float> master-peak-l: 0.0;
        in-out property <float> master-peak-r: 0.0;
        in-out property <float> master-volume: 1.0;
        in-out property <float> master-pan: 0.0;
        in-out property <bool> master-mute: false;
        in-out property <float> compressor-gr: 0.0;
        in-out property <image> eq-curve-image: @image-url("");
        in-out property <bool> mixer-visible: false;

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
        callback track-auto-param-clicked(string);
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
        callback track-input-mon-toggled(string);
        callback insert-pool-audio(string);
        callback copy-clips();
        callback paste-clips();
        callback delete-selected-clips();
        callback select-all-clips();
        callback delete-selected-bus();
        callback master-volume-changed(float);
        callback master-pan-changed(float);
        callback master-mute-toggled();
        callback add-send(string, string, bool);
        callback remove-send(string, string, bool);
        callback toggle-send-active(string, string, bool);
        callback toggle-send-pre-fader(string, string, bool);
        callback track-output-changed(string, string);
        callback bus-output-changed(string, string);

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

                    menu-transport := Rectangle {
                        min-width: 72px;
                        height: 28px;
                        background: root.open-menu == 4 ? #3a3a3a : transparent;
                        Text {
                            x: 10px;
                            y: (parent.height - 13px) / 2;
                            text: "Transport";
                            color: root.open-menu == 4 ? #ffffff : #cccccc;
                            font-size: 13px;
                        }
                        TouchArea { clicked => { root.open-menu = root.open-menu == 4 ? -1 : 4; } }
                    }

                    menu-view := Rectangle {
                        min-width: 48px;
                        height: 28px;
                        background: root.open-menu == 5 ? #3a3a3a : transparent;
                        Text {
                            x: 10px;
                            y: (parent.height - 13px) / 2;
                            text: "View";
                            color: root.open-menu == 5 ? #ffffff : #cccccc;
                            font-size: 13px;
                        }
                        TouchArea { clicked => { root.open-menu = root.open-menu == 5 ? -1 : 5; } }
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
                        width: 55px;
                        horizontal-stretch: 0;
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

                    Rectangle {
                        width: 45px;
                        horizontal-stretch: 0;
                        height: 24px;
                        background: root.pool-visible ? #1a3a5c : transparent;
                        border-radius: 3px;
                        Text {
                            x: 6px;
                            y: (parent.height - 13px) / 2;
                            text: "Pool";
                            color: root.pool-visible ? #64b5f6 : #888888;
                            font-size: 13px;
                        }
                        TouchArea { clicked => { root.pool-visible = !root.pool-visible; } }
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
                        y: 24px + track.index * 90px + 4px;
                        width: 216px;
                        height: 90px;
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
                            width: 142px;
                        }

                        Rectangle {
                            x: 154px;
                            y: 2px;
                            width: 40px;
                            height: 12px;
                            background: ta-trout.has-hover ? #333333 : #1a1a1a;
                            border-radius: 2px;
                            Text {
                                text: "=>" + track.output_name;
                                color: #888888;
                                font-size: 7px;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                            ta-trout := TouchArea { clicked => { root.track-output-changed(track.id, ""); } }
                        }

                        Rectangle {
                            x: 10px;
                            y: 16px;
                            width: 155px;
                            height: 8px;
                            background: #1a1a1a;
                            border-radius: 2px;

                            Rectangle {
                                x: 0px;
                                width: parent.width * track.volume;
                                height: 100%;
                                background: #4caf50;
                                border-radius: 2px;
                            }

                            Rectangle {
                                x: parent.width * track.volume - 1px;
                                width: 3px;
                                height: 100%;
                                background: #ffffff;
                                border-radius: 1px;
                            }

                            ta-tvol := TouchArea {
                                pointer-event(pe) => {
                                    if ta-tvol.pressed {
                                        let ratio = ta-tvol.mouse-x / ta-tvol.width;
                                        root.track-volume-changed(track.id, Math.max(0.0, Math.min(1.0, ratio)));
                                    }
                                }
                            }
                        }

                        Text {
                            x: 168px;
                            y: 16px;
                            width: 28px;
                            height: 8px;
                            text: track.volume > 0.001 ? "vol" : "-inf";
                            color: #888888;
                            font-size: 7px;
                            vertical-alignment: center;
                        }

                        Rectangle {
                            x: 10px;
                            y: 26px;
                            width: 155px;
                            height: 4px;
                            background: #1a1a1a;
                            border-radius: 2px;

                            Rectangle {
                                x: parent.width * 0.5 - 1px;
                                width: 1px;
                                height: 100%;
                                background: #333333;
                            }

                            Rectangle {
                                x: parent.width * 0.5 + parent.width * track.pan * 0.5 - 2px;
                                width: 5px;
                                height: 100%;
                                background: #64b5f6;
                                border-radius: 2px;
                            }

                            ta-tpan := TouchArea {
                                pointer-event(pe) => {
                                    if ta-tpan.pressed {
                                        let ratio = ta-tpan.mouse-x / ta-tpan.width;
                                        let pan = (ratio - 0.5) * 2.0;
                                        root.track-pan-changed(track.id, Math.max(-1.0, Math.min(1.0, pan)));
                                    }
                                }
                            }
                        }

                        Text {
                            x: 168px;
                            y: 26px;
                            width: 28px;
                            height: 4px;
                            text: track.pan < -0.01 ? "L" : track.pan > 0.01 ? "R" : "C";
                            color: #64b5f6;
                            font-size: 7px;
                            vertical-alignment: center;
                        }

                        Rectangle {
                            x: 200px;
                            y: 2px;
                            width: 6px;
                            height: 28px;
                            background: #1a1a1a;
                            border-radius: 2px;

                            Rectangle {
                                y: parent.height * (1.0 - track.peak_l);
                                width: parent.width;
                                height: parent.height * track.peak_l;
                                background: track.peak_l > 0.9 ? #ff4444 : track.peak_l > 0.7 ? #ffeb3b : #4caf50;
                                border-radius: 2px;
                            }
                        }

                        Rectangle {
                            x: 208px;
                            y: 2px;
                            width: 6px;
                            height: 28px;
                            background: #1a1a1a;
                            border-radius: 2px;

                            Rectangle {
                                y: parent.height * (1.0 - track.peak_r);
                                width: parent.width;
                                height: parent.height * track.peak_r;
                                background: track.peak_r > 0.9 ? #ff4444 : track.peak_r > 0.7 ? #ffeb3b : #4caf50;
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

                        Rectangle {
                            x: 0px;
                            y: 56px;
                            width: 100%;
                            height: 26px;
                            background: rgba(255, 255, 255, 0.03);
                            border-radius: 0px;
                            Text {
                                x: 4px;
                                y: (parent.height - 10px) / 2;
                                text: track.auto_param_name == "" ? "+A" : track.auto_param_name;
                                color: track.auto_param_name == "" ? #4caf50 : #aaaaaa;
                                font-size: 9px;
                            }
                            TouchArea { clicked => { root.track-auto-param-clicked(track.id); } }
                        }
                    }

                    for track in root.tracks: Rectangle {
                        y: 24px + (track.index + 1) * 90px;
                        width: 100%;
                        height: 1px;
                        background: #333333;
                    }

                    Rectangle {
                        y: 24px + root.track-count * 90px + 4px;
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
                        y: 24px + root.track-count * 90px + 28px + bus.index * 90px + 4px;
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
                            width: 142px;
                        }

                        Rectangle {
                            x: 154px;
                            y: 2px;
                            width: 40px;
                            height: 12px;
                            background: ta-brout.has-hover ? #333333 : #1a1a1a;
                            border-radius: 2px;
                            Text {
                                text: "=>" + bus.output_name;
                                color: #888888;
                                font-size: 7px;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                            ta-brout := TouchArea { clicked => { root.bus-output-changed(bus.id, ""); } }
                        }

                        Rectangle {
                            x: 10px;
                            y: 16px;
                            width: 155px;
                            height: 8px;
                            background: #1a1a1a;
                            border-radius: 2px;

                            Rectangle {
                                x: 0px;
                                width: parent.width * bus.volume;
                                height: 100%;
                                background: #2196f3;
                                border-radius: 2px;
                            }

                            Rectangle {
                                x: parent.width * bus.volume - 1px;
                                width: 3px;
                                height: 100%;
                                background: #ffffff;
                                border-radius: 1px;
                            }

                            ta-bvol := TouchArea {
                                pointer-event(pe) => {
                                    if ta-bvol.pressed {
                                        let ratio = ta-bvol.mouse-x / ta-bvol.width;
                                        root.bus-volume-changed(bus.id, Math.max(0.0, Math.min(1.0, ratio)));
                                    }
                                }
                            }
                        }

                        Rectangle {
                            x: 10px;
                            y: 26px;
                            width: 155px;
                            height: 4px;
                            background: #1a1a1a;
                            border-radius: 2px;

                            Rectangle {
                                x: parent.width * 0.5 - 1px;
                                width: 1px;
                                height: 100%;
                                background: #333333;
                            }

                            Rectangle {
                                x: parent.width * 0.5 + parent.width * bus.pan * 0.5 - 2px;
                                width: 5px;
                                height: 100%;
                                background: #64b5f6;
                                border-radius: 2px;
                            }

                            ta-bpan := TouchArea {
                                pointer-event(pe) => {
                                    if ta-bpan.pressed {
                                        let ratio = ta-bpan.mouse-x / ta-bpan.width;
                                        let pan = (ratio - 0.5) * 2.0;
                                        root.bus-pan-changed(bus.id, Math.max(-1.0, Math.min(1.0, pan)));
                                    }
                                }
                            }
                        }

                        Rectangle {
                            x: 200px;
                            y: 2px;
                            width: 6px;
                            height: 28px;
                            background: #1a1a1a;
                            border-radius: 2px;

                            Rectangle {
                                y: parent.height * (1.0 - bus.peak_l);
                                width: parent.width;
                                height: parent.height * bus.peak_l;
                                background: bus.peak_l > 0.9 ? #ff4444 : bus.peak_l > 0.7 ? #ffeb3b : #2196f3;
                                border-radius: 2px;
                            }
                        }

                        Rectangle {
                            x: 208px;
                            y: 2px;
                            width: 6px;
                            height: 28px;
                            background: #1a1a1a;
                            border-radius: 2px;

                            Rectangle {
                                y: parent.height * (1.0 - bus.peak_r);
                                width: parent.width;
                                height: parent.height * bus.peak_r;
                                background: bus.peak_r > 0.9 ? #ff4444 : bus.peak_r > 0.7 ? #ffeb3b : #2196f3;
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
                            y: 33px;
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
                        y: 24px + root.track-count * 90px + 28px + (bus.index + 1) * 90px;
                        width: 100%;
                        height: 1px;
                        background: #333333;
                    }

                    Rectangle {
                        y: 24px + root.track-count * 90px + 28px + root.buses.length * 90px + 4px;
                        width: 100%;
                        height: 22px;
                        background: #1a1a1a;
                        Text {
                            x: 8px;
                            y: (parent.height - 12px) / 2;
                            text: "MASTER";
                            color: #888888;
                            font-size: 11px;
                        }
                    }

                    Rectangle {
                        y: 24px + root.track-count * 90px + 28px + root.buses.length * 90px + 28px;
                        width: 216px;
                        height: 52px;
                        background: #2a2a2a;
                        border-radius: 3px;
                        x: 2px;
                        border-width: 1px;
                        border-color: #444444;

                        Text {
                            x: 10px;
                            y: 2px;
                            text: "Master";
                            color: #ff9800;
                            font-size: 11px;
                        }

                        Rectangle {
                            x: 10px;
                            y: 16px;
                            width: 155px;
                            height: 8px;
                            background: #1a1a1a;
                            border-radius: 2px;

                            Rectangle {
                                x: 0px;
                                width: parent.width * root.master-volume;
                                height: 100%;
                                background: #ff9800;
                                border-radius: 2px;
                            }

                            Rectangle {
                                x: parent.width * root.master-volume - 1px;
                                width: 3px;
                                height: 100%;
                                background: #ffffff;
                                border-radius: 1px;
                            }

                            ta-mvol := TouchArea {
                                pointer-event(pe) => {
                                    if ta-mvol.pressed {
                                        let ratio = ta-mvol.mouse-x / ta-mvol.width;
                                        root.master-volume-changed(Math.max(0.0, Math.min(1.0, ratio)));
                                    }
                                }
                            }
                        }

                        Rectangle {
                            x: 10px;
                            y: 26px;
                            width: 155px;
                            height: 4px;
                            background: #1a1a1a;
                            border-radius: 2px;

                            Rectangle {
                                x: parent.width * 0.5 - 1px;
                                width: 1px;
                                height: 100%;
                                background: #333333;
                            }

                            Rectangle {
                                x: parent.width * 0.5 + parent.width * root.master-pan * 0.5 - 2px;
                                width: 5px;
                                height: 100%;
                                background: #64b5f6;
                                border-radius: 2px;
                            }

                            ta-mpan := TouchArea {
                                pointer-event(pe) => {
                                    if ta-mpan.pressed {
                                        let ratio = ta-mpan.mouse-x / ta-mpan.width;
                                        let pan = (ratio - 0.5) * 2.0;
                                        root.master-pan-changed(Math.max(-1.0, Math.min(1.0, pan)));
                                    }
                                }
                            }
                        }

                        Rectangle {
                            x: 10px;
                            y: 33px;
                            width: 22px;
                            height: 16px;
                            background: root.master-mute ? #f44336 : #555555;
                            border-radius: 2px;
                            Text {
                                text: "M";
                                color: #ffffff;
                                font-size: 10px;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                            TouchArea { clicked => { root.master-mute-toggled(); } }
                        }

                        Rectangle {
                            x: 200px;
                            y: 2px;
                            width: 6px;
                            height: 48px;
                            background: #1a1a1a;
                            border-radius: 2px;

                            Rectangle {
                                y: parent.height * (1.0 - root.master-peak-l);
                                width: parent.width;
                                height: parent.height * root.master-peak-l;
                                background: root.master-peak-l > 0.9 ? #ff4444 : root.master-peak-l > 0.7 ? #ffeb3b : #ff9800;
                                border-radius: 2px;
                            }
                        }

                        Rectangle {
                            x: 208px;
                            y: 2px;
                            width: 6px;
                            height: 48px;
                            background: #1a1a1a;
                            border-radius: 2px;

                            Rectangle {
                                y: parent.height * (1.0 - root.master-peak-r);
                                width: parent.width;
                                height: parent.height * root.master-peak-r;
                                background: root.master-peak-r > 0.9 ? #ff4444 : root.master-peak-r > 0.7 ? #ffeb3b : #ff9800;
                                border-radius: 2px;
                            }
                        }
                    }
                }

                if root.pool-visible: Rectangle {
                    width: 280px;
                    background: #1e1e1e;
                    clip: true;

                    Rectangle {
                        width: 100%;
                        height: 24px;
                        background: #1a1a1a;
                        Text {
                            x: 8px;
                            y: (parent.height - 12px) / 2;
                            text: "AUDIO POOL";
                            color: #888888;
                            font-size: 11px;
                        }
                        Rectangle {
                            x: parent.width - 24px;
                            y: 4px;
                            width: 16px;
                            height: 16px;
                            background: ta-close-pool.has-hover ? #553333 : transparent;
                            border-radius: 2px;
                            Text { text: "x"; color: #aa6666; font-size: 10px; horizontal-alignment: center; vertical-alignment: center; }
                            ta-close-pool := TouchArea { clicked => { root.pool-visible = false; } }
                        }
                    }

                    Rectangle {
                        y: 24px;
                        width: 100%;
                        height: parent.height - 24px;
                        clip: true;

                        VerticalLayout {
                            padding: 2px;
                            spacing: 1px;

                            for entry in root.pool-entries: Rectangle {
                                height: 36px;
                                background: ta-pool-item.has-hover ? #2a3a5a : transparent;
                                border-radius: 3px;

                                VerticalLayout {
                                    padding: 2px;
                                    Text {
                                        x: 4px;
                                        text: entry.name;
                                        color: #cccccc;
                                        font-size: 10px;
                                        overflow: elide;
                                    }
                                    Text {
                                        x: 4px;
                                        text: entry.info;
                                        color: #777777;
                                        font-size: 8px;
                                    }
                                }
                                ta-pool-item := TouchArea { clicked => { root.insert-pool-audio(entry.path); } }
                            }
                        }
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
                            text: root.bpm-display;
                            color: #aaaaaa;
                            font-size: 12px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        Text {
                            text: root.time-sig-display;
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
                            y: clip.track_index * 90px + 4px;
                            height: 82px;
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

                            Rectangle {
                                y: 56px;
                                width: 100%;
                                height: 26px;
                                background: rgba(255, 255, 255, 0.03);
                                border-radius: 3px;
                                clip: true;
                                Image {
                                    width: 100%;
                                    height: 100%;
                                    source: clip.auto_image;
                                    image-fit: fill;
                                }
                                Text {
                                    x: 2px;
                                    y: 2px;
                                    font-size: 8px;
                                    color: rgba(255, 255, 255, 0.35);
                                    text: clip.auto_param_name;
                                }
                            }
                        }

                        for tick in root.ruler-ticks: Rectangle {
                            x: tick.position - root.timeline-scroll-x;
                            width: 1px;
                            height: 100%;
                            background: rgba(255, 255, 255, 0.05);
                            visible: root.snap-enabled;
                        }

                        for track in root.tracks: Rectangle {
                            y: (track.index + 1) * 90px;
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

                            HorizontalLayout {
                                Text {
                                    text: root.effect-editor-title;
                                    color: #cccccc;
                                    font-size: 12px;
                                    vertical-alignment: center;
                                }

                                Rectangle {
                                    horizontal-stretch: 1;
                                }

                                Rectangle {
                                    min-width: 50px;
                                    height: 18px;
                                    background: ta-bypass.has-hover ? #553333 : #444444;
                                    border-radius: 2px;
                                    Text {
                                        text: "Bypass";
                                        color: #cc8888;
                                        font-size: 9px;
                                        horizontal-alignment: center;
                                        vertical-alignment: center;
                                    }
                                    ta-bypass := TouchArea {
                                        clicked => {
                                            root.toggle-effect-bypass(root.selected-effect-target, root.selected-effect-is-track, root.selected-effect-index);
                                        }
                                    }
                                }
                            }

                            if root.effect-editor-title == "Equalizer": Rectangle {
                                height: 80px;
                                background: #0a0a0a;
                                border-radius: 2px;

                                Image {
                                    x: 0px;
                                    y: 0px;
                                    width: 100%;
                                    height: 100%;
                                    source: root.eq-curve-image;
                                    image-fit: fill;
                                }
                            }

                            if root.effect-editor-title == "Compressor": Rectangle {
                                height: 10px;
                                background: #1a1a1a;
                                border-radius: 2px;

                                Rectangle {
                                    x: parent.width * (1.0 + root.compressor-gr / 60.0);
                                    width: parent.width * Math.max(0.0, -root.compressor-gr / 60.0);
                                    height: 100%;
                                    background: #ff9800;
                                    border-radius: 2px;
                                }
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

                if root.mixer-visible: Rectangle {
                    width: 200px;
                    background: #222222;

                    Rectangle {
                        width: 100%;
                        height: 24px;
                        background: #1a1a1a;
                        Text {
                            x: 8px;
                            y: (parent.height - 12px) / 2;
                            text: "MIXER";
                            color: #888888;
                            font-size: 11px;
                        }
                        Rectangle {
                            x: parent.width - 24px;
                            y: 4px;
                            width: 16px;
                            height: 16px;
                            background: ta-close-mixer.has-hover ? #553333 : transparent;
                            border-radius: 2px;
                            Text { text: "x"; color: #aa6666; font-size: 10px; horizontal-alignment: center; vertical-alignment: center; }
                            ta-close-mixer := TouchArea { clicked => { root.mixer-visible = false; } }
                        }
                    }

                    Rectangle {
                        y: 24px;
                        width: 100%;
                        height: parent.height - 24px;
                        clip: true;

                        HorizontalLayout {
                            spacing: 2px;
                            padding: 4px;

                            for track in root.tracks: Rectangle {
                                width: 70px;
                                height: 100%;
                                background: track.selected ? #2a3a5a : #2a2a2a;
                                border-radius: 3px;

                                VerticalLayout {
                                    padding: 4px;
                                    spacing: 2px;

                                    Text { text: track.label; color: #cccccc; font-size: 9px; horizontal-alignment: center; overflow: elide; }

                                    Rectangle {
                                        height: 80px; width: 24px;
                                        x: (parent.width - 24px) / 2;
                                        background: #1a1a1a;
                                        border-radius: 2px;

                                        Rectangle {
                                            y: parent.height * (1.0 - track.volume);
                                            width: parent.width;
                                            height: parent.height * track.volume;
                                            background: #4caf50;
                                            border-radius: 2px;
                                        }

                                        Rectangle {
                                            y: parent.height * (1.0 - track.volume) - 2px;
                                            width: parent.width; height: 4px;
                                            background: #ffffff;
                                            border-radius: 1px;
                                        }

                                        ta-mx-tvol := TouchArea {
                                            pointer-event(pe) => {
                                                if ta-mx-tvol.pressed {
                                                    let ratio = 1.0 - ta-mx-tvol.mouse-y / ta-mx-tvol.height;
                                                    root.track-volume-changed(track.id, Math.max(0.0, Math.min(1.0, ratio)));
                                                }
                                            }
                                        }
                                    }

                                    Rectangle {
                                        height: 8px; width: 40px;
                                        x: (parent.width - 40px) / 2;
                                        background: #1a1a1a;
                                        border-radius: 2px;

                                        Rectangle {
                                            x: parent.width * 0.5 + parent.width * track.pan * 0.5 - 3px;
                                            width: 6px; height: 100%;
                                            background: #64b5f6;
                                            border-radius: 2px;
                                        }

                                        ta-mx-tpan := TouchArea {
                                            pointer-event(pe) => {
                                                if ta-mx-tpan.pressed {
                                                    let ratio = ta-mx-tpan.mouse-x / ta-mx-tpan.width;
                                                    let pan = (ratio - 0.5) * 2.0;
                                                    root.track-pan-changed(track.id, Math.max(-1.0, Math.min(1.0, pan)));
                                                }
                                            }
                                        }
                                    }

                                    Rectangle {
                                        height: 60px; width: 14px;
                                        x: (parent.width - 14px) / 2;
                                        background: #1a1a1a;
                                        border-radius: 1px;

                                        Rectangle {
                                            y: parent.height * (1.0 - track.peak_l);
                                            width: 6px;
                                            height: parent.height * track.peak_l;
                                            background: track.peak_l > 0.9 ? #ff4444 : track.peak_l > 0.7 ? #ffeb3b : #4caf50;
                                            border-radius: 1px;
                                        }

                                        Rectangle {
                                            x: 7px;
                                            y: parent.height * (1.0 - track.peak_r);
                                            width: 6px;
                                            height: parent.height * track.peak_r;
                                            background: track.peak_r > 0.9 ? #ff4444 : track.peak_r > 0.7 ? #ffeb3b : #4caf50;
                                            border-radius: 1px;
                                        }
                                    }

                                    HorizontalLayout {
                                        spacing: 2px; height: 16px;
                                        Rectangle { width: 18px; height: 14px; background: track.mute ? #f44336 : #555555; border-radius: 2px; Text { text: "M"; color: #ffffff; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; } TouchArea { clicked => { root.track-mute-toggled(track.id); } } }
                                        Rectangle { width: 18px; height: 14px; background: track.solo ? #ffeb3b : #555555; border-radius: 2px; Text { text: "S"; color: #000000; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; } TouchArea { clicked => { root.track-solo-toggled(track.id); } } }
                                    }

                                    for fx in track.effects: Rectangle {
                                        height: 14px; min-width: 16px;
                                        background: fx.bypassed ? #333333 : #444444;
                                        border-radius: 2px;
                                        Text { text: fx.name; color: fx.bypassed ? #666666 : #cccccc; font-size: 7px; horizontal-alignment: center; vertical-alignment: center; }
                                        TouchArea { clicked => { root.effect-selected(track.id, fx.idx, true); } }
                                    }
                                }
                            }

                            for bus in root.buses: Rectangle {
                                width: 70px; height: 100%;
                                background: bus.selected ? #2a3a5a : #2a2a2a;
                                border-radius: 3px;

                                VerticalLayout {
                                    padding: 4px; spacing: 2px;
                                    Text { text: bus.label; color: #cccccc; font-size: 9px; horizontal-alignment: center; overflow: elide; }

                                    Rectangle {
                                        height: 80px; width: 24px;
                                        x: (parent.width - 24px) / 2;
                                        background: #1a1a1a; border-radius: 2px;

                                        Rectangle { y: parent.height * (1.0 - bus.volume); width: parent.width; height: parent.height * bus.volume; background: #2196f3; border-radius: 2px; }
                                        Rectangle { y: parent.height * (1.0 - bus.volume) - 2px; width: parent.width; height: 4px; background: #ffffff; border-radius: 1px; }

                                        ta-mx-bvol := TouchArea {
                                            pointer-event(pe) => {
                                                if ta-mx-bvol.pressed {
                                                    let ratio = 1.0 - ta-mx-bvol.mouse-y / ta-mx-bvol.height;
                                                    root.bus-volume-changed(bus.id, Math.max(0.0, Math.min(1.0, ratio)));
                                                }
                                            }
                                        }
                                    }

                                    Rectangle { height: 8px; width: 40px; x: (parent.width - 40px) / 2; background: #1a1a1a; border-radius: 2px;
                                        Rectangle { x: parent.width * 0.5 + parent.width * bus.pan * 0.5 - 3px; width: 6px; height: 100%; background: #64b5f6; border-radius: 2px; }
                                        ta-mx-bpan := TouchArea {
                                            pointer-event(pe) => {
                                                if ta-mx-bpan.pressed {
                                                    let ratio = ta-mx-bpan.mouse-x / ta-mx-bpan.width;
                                                    let pan = (ratio - 0.5) * 2.0;
                                                    root.bus-pan-changed(bus.id, Math.max(-1.0, Math.min(1.0, pan)));
                                                }
                                            }
                                        }
                                    }

                                    Rectangle { height: 60px; width: 14px; x: (parent.width - 14px) / 2; background: #1a1a1a; border-radius: 1px;
                                        Rectangle { y: parent.height * (1.0 - bus.peak_l); width: 6px; height: parent.height * bus.peak_l; background: bus.peak_l > 0.9 ? #ff4444 : bus.peak_l > 0.7 ? #ffeb3b : #2196f3; border-radius: 1px; }
                                        Rectangle { x: 7px; y: parent.height * (1.0 - bus.peak_r); width: 6px; height: parent.height * bus.peak_r; background: bus.peak_r > 0.9 ? #ff4444 : bus.peak_r > 0.7 ? #ffeb3b : #2196f3; border-radius: 1px; }
                                    }

                                    HorizontalLayout { spacing: 2px; height: 16px;
                                        Rectangle { width: 18px; height: 14px; background: bus.mute ? #f44336 : #555555; border-radius: 2px; Text { text: "M"; color: #ffffff; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; } TouchArea { clicked => { root.bus-mute-toggled(bus.id); } } }
                                        Rectangle { width: 18px; height: 14px; background: bus.solo ? #ffeb3b : #555555; border-radius: 2px; Text { text: "S"; color: #000000; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; } TouchArea { clicked => { root.bus-solo-toggled(bus.id); } } }
                                    }
                                }
                            }

                            Rectangle { width: 70px; height: 100%; background: #2a2a2a; border-radius: 3px; border-width: 1px; border-color: #444444;
                                VerticalLayout { padding: 4px; spacing: 2px;
                                    Text { text: "Master"; color: #ff9800; font-size: 9px; horizontal-alignment: center; }
                                    Rectangle { height: 80px; width: 24px; x: (parent.width - 24px) / 2; background: #1a1a1a; border-radius: 2px;
                                        Rectangle { y: parent.height * (1.0 - root.master-volume); width: parent.width; height: parent.height * root.master-volume; background: #ff9800; border-radius: 2px; }
                                        Rectangle { y: parent.height * (1.0 - root.master-volume) - 2px; width: parent.width; height: 4px; background: #ffffff; border-radius: 1px; }
                                        ta-mx-mvol := TouchArea {
                                            pointer-event(pe) => {
                                                if ta-mx-mvol.pressed {
                                                    let ratio = 1.0 - ta-mx-mvol.mouse-y / ta-mx-mvol.height;
                                                    root.master-volume-changed(Math.max(0.0, Math.min(1.0, ratio)));
                                                }
                                            }
                                        }
                                    }
                                    Rectangle { height: 8px; width: 40px; x: (parent.width - 40px) / 2; background: #1a1a1a; border-radius: 2px;
                                        Rectangle { x: parent.width * 0.5 + parent.width * root.master-pan * 0.5 - 3px; width: 6px; height: 100%; background: #64b5f6; border-radius: 2px; }
                                        ta-mx-mpan := TouchArea {
                                            pointer-event(pe) => {
                                                if ta-mx-mpan.pressed {
                                                    let ratio = ta-mx-mpan.mouse-x / ta-mx-mpan.width;
                                                    let pan = (ratio - 0.5) * 2.0;
                                                    root.master-pan-changed(Math.max(-1.0, Math.min(1.0, pan)));
                                                }
                                            }
                                        }
                                    }
                                    Rectangle { height: 60px; width: 14px; x: (parent.width - 14px) / 2; background: #1a1a1a; border-radius: 1px;
                                        Rectangle { y: parent.height * (1.0 - root.master-peak-l); width: 6px; height: parent.height * root.master-peak-l; background: root.master-peak-l > 0.9 ? #ff4444 : root.master-peak-l > 0.7 ? #ffeb3b : #ff9800; border-radius: 1px; }
                                        Rectangle { x: 7px; y: parent.height * (1.0 - root.master-peak-r); width: 6px; height: parent.height * root.master-peak-r; background: root.master-peak-r > 0.9 ? #ff4444 : root.master-peak-r > 0.7 ? #ffeb3b : #ff9800; border-radius: 1px; }
                                    }
                                    Rectangle { width: 18px; height: 14px; x: (parent.width - 18px) / 2; background: root.master-mute ? #f44336 : #555555; border-radius: 2px; Text { text: "M"; color: #ffffff; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; } TouchArea { clicked => { root.master-mute-toggled(); } } }
                                }
                            }
                        }
                    }
                }
                }
            }

            if root.open-menu == 4: Rectangle {
                x: menu-transport.x;
                y: menu-transport.y + menu-transport.height;
                width: 170px;
                height: 200px;
                background: #2a2a2a;
                border-width: 1px;
                border-color: #444444;
                drop-shadow-blur: 8px;
                drop-shadow-color: #000000;

                VerticalLayout {
                    padding: 4px;
                    spacing: 0px;

                    Rectangle { height: 26px; background: ta-tp-play.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout { padding-left: 12px; Text { text: "▶ Play"; color: #4caf50; font-size: 12px; vertical-alignment: center; } Rectangle { horizontal-stretch: 1; } Text { text: "Space"; color: #666666; font-size: 11px; vertical-alignment: center; padding-right: 8px; } }
                        ta-tp-play := TouchArea { clicked => { root.open-menu = -1; root.play(); } }
                    }
                    Rectangle { height: 26px; background: ta-tp-stop.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout { padding-left: 12px; Text { text: "■ Stop"; color: #f44336; font-size: 12px; vertical-alignment: center; } }
                        ta-tp-stop := TouchArea { clicked => { root.open-menu = -1; root.stop(); } }
                    }
                    Rectangle { height: 26px; background: ta-tp-rec.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout { padding-left: 12px; Text { text: "● Record"; color: #ff6666; font-size: 12px; vertical-alignment: center; } Rectangle { horizontal-stretch: 1; } Text { text: "R"; color: #666666; font-size: 11px; vertical-alignment: center; padding-right: 8px; } }
                        ta-tp-rec := TouchArea { clicked => { root.open-menu = -1; root.start-recording(); } }
                    }
                    Rectangle { height: 1px; background: #444444; }
                    Rectangle { height: 26px; background: ta-tp-loop.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout { padding-left: 12px; Text { text: "Loop"; color: #cccccc; font-size: 12px; vertical-alignment: center; } Rectangle { horizontal-stretch: 1; } Text { text: "L"; color: #666666; font-size: 11px; vertical-alignment: center; padding-right: 8px; } }
                        ta-tp-loop := TouchArea { clicked => { root.open-menu = -1; root.toggle-loop(); } }
                    }
                    Rectangle { height: 26px; background: ta-tp-start.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout { padding-left: 12px; Text { text: "Go to Start"; color: #cccccc; font-size: 12px; vertical-alignment: center; } Rectangle { horizontal-stretch: 1; } Text { text: "Home"; color: #666666; font-size: 11px; vertical-alignment: center; padding-right: 8px; } }
                        ta-tp-start := TouchArea { clicked => { root.open-menu = -1; root.go-to-start(); } }
                    }
                    Rectangle { height: 26px; background: ta-tp-end.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout { padding-left: 12px; Text { text: "Go to End"; color: #cccccc; font-size: 12px; vertical-alignment: center; } Rectangle { horizontal-stretch: 1; } Text { text: "End"; color: #666666; font-size: 11px; vertical-alignment: center; padding-right: 8px; } }
                        ta-tp-end := TouchArea { clicked => { root.open-menu = -1; root.go-to-end(); } }
                    }
                }
            }

            if root.open-menu == 5: Rectangle {
                x: menu-view.x;
                y: menu-view.y + menu-view.height;
                width: 180px;
                height: 140px;
                background: #2a2a2a;
                border-width: 1px;
                border-color: #444444;
                drop-shadow-blur: 8px;
                drop-shadow-color: #000000;

                VerticalLayout {
                    padding: 4px;
                    spacing: 0px;

                    Rectangle { height: 26px; background: ta-vw-pool.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout { padding-left: 12px; Text { text: root.pool-visible ? "✓ Audio Pool" : "   Audio Pool"; color: #cccccc; font-size: 12px; vertical-alignment: center; } Rectangle { horizontal-stretch: 1; } Text { text: "P"; color: #666666; font-size: 11px; vertical-alignment: center; padding-right: 8px; } }
                        ta-vw-pool := TouchArea { clicked => { root.open-menu = -1; root.pool-visible = !root.pool-visible; } }
                    }
                    Rectangle { height: 26px; background: ta-vw-snap.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout { padding-left: 12px; Text { text: root.snap-enabled ? "✓ Snap Enabled" : "   Snap Enabled"; color: #cccccc; font-size: 12px; vertical-alignment: center; } Rectangle { horizontal-stretch: 1; } Text { text: "N"; color: #666666; font-size: 11px; vertical-alignment: center; padding-right: 8px; } }
                        ta-vw-snap := TouchArea { clicked => { root.open-menu = -1; root.snap-enabled = !root.snap-enabled; } }
                    }
                    Rectangle { height: 1px; background: #444444; }
                    Rectangle { height: 26px; background: ta-vw-mixer.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout { padding-left: 12px; Text { text: root.mixer-visible ? "✓ Show Mixer" : "   Show Mixer"; color: #cccccc; font-size: 12px; vertical-alignment: center; } }
                        ta-vw-mixer := TouchArea { clicked => { root.open-menu = -1; root.mixer-visible = !root.mixer-visible; } }
                    }
                    Rectangle { height: 26px; background: ta-vw-snapcfg.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout { padding-left: 12px; Text { text: "Snap Config..."; color: #cccccc; font-size: 12px; vertical-alignment: center; } }
                        ta-vw-snapcfg := TouchArea { clicked => { root.open-menu = -1; root.snap-menu-open = true; } }
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
                height: 168px;
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
                    Rectangle { height: 1px; background: #444444; }
                    Rectangle {
                        height: 26px;
                        background: ta-edit-copy.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout {
                            padding-left: 12px;
                            Text {
                                text: "Copy";
                                color: #cccccc;
                                font-size: 12px;
                                vertical-alignment: center;
                            }
                            Rectangle { horizontal-stretch: 1; }
                            Text {
                                text: "Ctrl+C";
                                color: #666666;
                                font-size: 10px;
                                vertical-alignment: center;
                                padding-right: 8px;
                            }
                        }
                        ta-edit-copy := TouchArea { clicked => { root.open-menu = -1; root.copy-clips(); } }
                    }
                    Rectangle {
                        height: 26px;
                        background: ta-edit-paste.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout {
                            padding-left: 12px;
                            Text {
                                text: "Paste";
                                color: #cccccc;
                                font-size: 12px;
                                vertical-alignment: center;
                            }
                            Rectangle { horizontal-stretch: 1; }
                            Text {
                                text: "Ctrl+V";
                                color: #666666;
                                font-size: 10px;
                                vertical-alignment: center;
                                padding-right: 8px;
                            }
                        }
                        ta-edit-paste := TouchArea { clicked => { root.open-menu = -1; root.paste-clips(); } }
                    }
                    Rectangle { height: 1px; background: #444444; }
                    Rectangle {
                        height: 26px;
                        background: ta-edit-delete.has-hover ? #553333 : transparent;
                        HorizontalLayout {
                            padding-left: 12px;
                            Text {
                                text: "Delete";
                                color: #cc8888;
                                font-size: 12px;
                                vertical-alignment: center;
                            }
                            Rectangle { horizontal-stretch: 1; }
                            Text {
                                text: "Del";
                                color: #666666;
                                font-size: 10px;
                                vertical-alignment: center;
                                padding-right: 8px;
                            }
                        }
                        ta-edit-delete := TouchArea { clicked => { root.open-menu = -1; root.delete-selected-clips(); } }
                    }
                    Rectangle {
                        height: 26px;
                        background: ta-edit-selall.has-hover ? #3a5a8a : transparent;
                        HorizontalLayout {
                            padding-left: 12px;
                            Text {
                                text: "Select All";
                                color: #cccccc;
                                font-size: 12px;
                                vertical-alignment: center;
                            }
                            Rectangle { horizontal-stretch: 1; }
                            Text {
                                text: "Ctrl+A";
                                color: #666666;
                                font-size: 10px;
                                vertical-alignment: center;
                                padding-right: 8px;
                            }
                        }
                        ta-edit-selall := TouchArea { clicked => { root.open-menu = -1; root.select-all-clips(); } }
                    }
                }
            }

            if root.open-menu == 3: Rectangle {
                x: menu-track.x;
                y: menu-track.y + menu-track.height;
                width: 160px;
                height: 90px;
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
                    Rectangle { height: 1px; background: #444444; }
                    Rectangle {
                        height: 26px;
                        background: root.selected-bus-id != "" && ta-del-bus.has-hover ? #553333 : transparent;
                        HorizontalLayout {
                            padding-left: 12px;
                            Text {
                                text: "Delete Bus";
                                color: root.selected-bus-id != "" ? #cc8888 : #666666;
                                font-size: 12px;
                                vertical-alignment: center;
                            }
                        }
                        ta-del-bus := TouchArea { clicked => { root.open-menu = -1; if root.selected-bus-id != "" { root.delete-selected-bus(); } } }
                    }
                }
            }

            if root.snap-menu-open: Rectangle {
                x: 0px;
                y: 0px;
                width: 100%;
                height: 100%;
                background: rgba(0, 0, 0, 0.5);

                Rectangle {
                    x: (parent.width - 240px) / 2;
                    y: (parent.height - 200px) / 2;
                    width: 240px;
                    height: 200px;
                    background: #2a2a2a;
                    border-width: 1px;
                    border-color: #444444;
                    drop-shadow-blur: 12px;
                    drop-shadow-color: #000000;

                    VerticalLayout {
                        padding: 8px;
                        spacing: 4px;

                        HorizontalLayout {
                            Text {
                                text: "Snap Settings";
                                color: #cccccc;
                                font-size: 13px;
                                vertical-alignment: center;
                            }
                            Rectangle { horizontal-stretch: 1; }
                            Rectangle {
                                width: 18px;
                                height: 18px;
                                background: ta-close-snap.has-hover ? #553333 : transparent;
                                border-radius: 2px;
                                Text { text: "x"; color: #aa6666; font-size: 10px; horizontal-alignment: center; vertical-alignment: center; }
                                ta-close-snap := TouchArea { clicked => { root.snap-menu-open = false; } }
                            }
                        }

                        Rectangle { height: 1px; background: #444444; }

                        Text { text: "Snap Mode"; color: #888888; font-size: 10px; }
                        HorizontalLayout {
                            Rectangle {
                                height: 22px;
                                width: 55px;
                                background: root.snap-mode == 0 ? #3a5a8a : #333333;
                                border-radius: 2px;
                                Text { text: "Adaptive"; color: root.snap-mode == 0 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                TouchArea { clicked => { root.snap-mode = 0; } }
                            }
                            Rectangle {
                                height: 22px;
                                width: 50px;
                                background: root.snap-mode == 1 ? #3a5a8a : #333333;
                                border-radius: 2px;
                                Text { text: "Beats"; color: root.snap-mode == 1 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                TouchArea { clicked => { root.snap-mode = 1; } }
                            }
                            Rectangle {
                                height: 22px;
                                width: 50px;
                                background: root.snap-mode == 2 ? #3a5a8a : #333333;
                                border-radius: 2px;
                                Text { text: "Time"; color: root.snap-mode == 2 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                TouchArea { clicked => { root.snap-mode = 2; } }
                            }
                            Rectangle {
                                height: 22px;
                                width: 55px;
                                background: root.snap-mode == 3 ? #3a5a8a : #333333;
                                border-radius: 2px;
                                Text { text: "Frames"; color: root.snap-mode == 3 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                TouchArea { clicked => { root.snap-mode = 3; } }
                            }
                        }

                        if root.snap-mode == 1: Rectangle {
                            height: 30px;
                            Text { y: 4px; text: "Beat Division"; color: #888888; font-size: 10px; }
                            HorizontalLayout {
                                y: 16px; spacing: 2px;
                                Rectangle {
                                    height: 14px; width: 24px;
                                    background: root.snap-param == 0 ? #3a5a8a : #333333; border-radius: 2px;
                                    Text { text: "1"; color: root.snap-param == 0 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                    TouchArea { clicked => { root.snap-param = 0; } }
                                }
                                Rectangle {
                                    height: 14px; width: 24px;
                                    background: root.snap-param == 1 ? #3a5a8a : #333333; border-radius: 2px;
                                    Text { text: "1/2"; color: root.snap-param == 1 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                    TouchArea { clicked => { root.snap-param = 1; } }
                                }
                                Rectangle {
                                    height: 14px; width: 24px;
                                    background: root.snap-param == 2 ? #3a5a8a : #333333; border-radius: 2px;
                                    Text { text: "1/4"; color: root.snap-param == 2 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                    TouchArea { clicked => { root.snap-param = 2; } }
                                }
                                Rectangle {
                                    height: 14px; width: 24px;
                                    background: root.snap-param == 3 ? #3a5a8a : #333333; border-radius: 2px;
                                    Text { text: "1/8"; color: root.snap-param == 3 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                    TouchArea { clicked => { root.snap-param = 3; } }
                                }
                                Rectangle {
                                    height: 14px; width: 24px;
                                    background: root.snap-param == 4 ? #3a5a8a : #333333; border-radius: 2px;
                                    Text { text: "1/16"; color: root.snap-param == 4 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                    TouchArea { clicked => { root.snap-param = 4; } }
                                }
                                Rectangle {
                                    height: 14px; width: 24px;
                                    background: root.snap-param == 5 ? #3a5a8a : #333333; border-radius: 2px;
                                    Text { text: "1/32"; color: root.snap-param == 5 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                    TouchArea { clicked => { root.snap-param = 5; } }
                                }
                            }
                        }

                        if root.snap-mode == 2: Rectangle {
                            height: 30px;
                            Text { y: 4px; text: "Time Resolution"; color: #888888; font-size: 10px; }
                            HorizontalLayout {
                                y: 16px; spacing: 2px;
                                Rectangle {
                                    height: 14px; width: 36px;
                                    background: root.snap-param == 0 ? #3a5a8a : #333333; border-radius: 2px;
                                    Text { text: "100ms"; color: root.snap-param == 0 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                    TouchArea { clicked => { root.snap-param = 0; } }
                                }
                                Rectangle {
                                    height: 14px; width: 36px;
                                    background: root.snap-param == 1 ? #3a5a8a : #333333; border-radius: 2px;
                                    Text { text: "250ms"; color: root.snap-param == 1 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                    TouchArea { clicked => { root.snap-param = 1; } }
                                }
                                Rectangle {
                                    height: 14px; width: 36px;
                                    background: root.snap-param == 2 ? #3a5a8a : #333333; border-radius: 2px;
                                    Text { text: "500ms"; color: root.snap-param == 2 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                    TouchArea { clicked => { root.snap-param = 2; } }
                                }
                                Rectangle {
                                    height: 14px; width: 36px;
                                    background: root.snap-param == 3 ? #3a5a8a : #333333; border-radius: 2px;
                                    Text { text: "1s"; color: root.snap-param == 3 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                    TouchArea { clicked => { root.snap-param = 3; } }
                                }
                            }
                        }

                        if root.snap-mode == 3: Rectangle {
                            height: 30px;
                            Text { y: 4px; text: "Frame Rate"; color: #888888; font-size: 10px; }
                            HorizontalLayout {
                                y: 16px; spacing: 2px;
                                Rectangle {
                                    height: 14px; width: 28px;
                                    background: root.snap-param == 0 ? #3a5a8a : #333333; border-radius: 2px;
                                    Text { text: "24"; color: root.snap-param == 0 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                    TouchArea { clicked => { root.snap-param = 0; } }
                                }
                                Rectangle {
                                    height: 14px; width: 28px;
                                    background: root.snap-param == 1 ? #3a5a8a : #333333; border-radius: 2px;
                                    Text { text: "25"; color: root.snap-param == 1 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                    TouchArea { clicked => { root.snap-param = 1; } }
                                }
                                Rectangle {
                                    height: 14px; width: 28px;
                                    background: root.snap-param == 2 ? #3a5a8a : #333333; border-radius: 2px;
                                    Text { text: "30"; color: root.snap-param == 2 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                    TouchArea { clicked => { root.snap-param = 2; } }
                                }
                                Rectangle {
                                    height: 14px; width: 36px;
                                    background: root.snap-param == 3 ? #3a5a8a : #333333; border-radius: 2px;
                                    Text { text: "30D"; color: root.snap-param == 3 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                    TouchArea { clicked => { root.snap-param = 3; } }
                                }
                                Rectangle {
                                    height: 14px; width: 28px;
                                    background: root.snap-param == 4 ? #3a5a8a : #333333; border-radius: 2px;
                                    Text { text: "60"; color: root.snap-param == 4 ? #ffffff : #888888; font-size: 8px; horizontal-alignment: center; vertical-alignment: center; }
                                    TouchArea { clicked => { root.snap-param = 4; } }
                                }
                            }
                        }
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
