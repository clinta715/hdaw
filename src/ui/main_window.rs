use slint::ComponentHandle;

slint::slint! {
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
    }

    export struct TrackInfo {
        id: string,
        index: int,
        label: string,
    }

    export component MainWindow {
        in-out property <bool> can-undo: false;
        in-out property <bool> can-redo: false;
        in-out property <[ClipInfo]> clips;
        in-out property <[TrackInfo]> tracks;
        in-out property <length> pixels-per-second: 50px;

        callback undo();
        callback redo();
        callback timeline-clicked();
        callback zoom-in();
        callback zoom-out();

        width: 1280px;
        height: 720px;

        VerticalLayout {
            Rectangle {
                height: 28px;
                background: #252525;

                HorizontalLayout {
                    padding-left: 8px;
                    spacing: 16px;
                    Text { text: "File"; color: #cccccc; font-size: 13px; }
                    Text { text: "Edit"; color: #cccccc; font-size: 13px; }
                    Text { text: "View"; color: #cccccc; font-size: 13px; }
                    Text { text: "Track"; color: #cccccc; font-size: 13px; }
                    Text { text: "Transport"; color: #cccccc; font-size: 13px; }
                    Text { text: "Help"; color: #cccccc; font-size: 13px; }
                }
            }

            Rectangle {
                height: 32px;
                background: #2a2a2a;
                HorizontalLayout {
                    padding-left: 8px;
                    spacing: 4px;
                    Text { text: "[<<]"; color: #888888; font-size: 14px; }
                    Text { text: "[>>]"; color: #888888; font-size: 14px; }
                    Text { text: "[Play]"; color: #4caf50; font-size: 14px; }
                    Text { text: "[Stop]"; color: #f44336; font-size: 14px; }
                    Text { text: "[Loop]"; color: #888888; font-size: 14px; }
                    Text { text: "  |  "; color: #444444; font-size: 14px; }
                    Text { text: "00.00.00.000"; color: #8bc34a; font-size: 14px; }
                    Text { text: "  |  "; color: #444444; font-size: 14px; }
                    Text { text: "120.0 BPM"; color: #aaaaaa; font-size: 14px; }
                    Text { text: "4/4"; color: #aaaaaa; font-size: 14px; }
                }
            }

            HorizontalLayout {
                Rectangle {
                    horizontal-stretch: 3;
                    background: #1a1a1a;
                    clip: true;

                    ta := TouchArea {
                        clicked => {
                            root.timeline-clicked();
                        }
                        scroll-event(event) => {
                            root.zoom-in();
                            accept
                        }
                    }

                    for clip in root.clips: Rectangle {
                        x: clip.x;
                        width: max(clip.width, 4px);
                        y: clip.track_index * 60px + 4px;
                        height: 52px;
                        background: clip.color;
                        border-radius: 3px;
                        border-width: 1px;
                        border-color: clip.selected ? #ffffff : #333333;

                        Rectangle {
                            x: 0px;
                            width: clip.fade_in_width;
                            height: 100%;
                            background: rgba(0, 0, 0, 0.3);
                        }

                        Rectangle {
                            x: parent.width - clip.fade_out_width;
                            width: clip.fade_out_width;
                            height: 100%;
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
                    }

                    for track in root.tracks: Rectangle {
                        y: (track.index + 1) * 60px;
                        width: 100%;
                        height: 1px;
                        background: #333333;
                    }
                }

                Rectangle {
                    horizontal-stretch: 1;
                    background: #222222;
                    min-width: 200px;
                }
            }

            Rectangle {
                height: 120px;
                background: #1e1e1e;
            }
        }
    }
}

pub fn run() {
    let window = MainWindow::new().unwrap();
    window.run().unwrap();
}
