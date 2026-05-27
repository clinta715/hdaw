import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import com.hdaw 1.0

ApplicationWindow {
    id: mainWindow
    visible: true
    width: 1280
    height: 720
    title: "HDAW"
    minimumWidth: 800
    minimumHeight: 500

    TransportBar { id: transport }
    MixerModel { id: mixer }
    StateBridge { id: state }
    ShortcutHandler { id: sc }
    TimelineModel { id: tl }

    Shortcut { sequence: "Space";     onActivated: transport.toggle_play_stop() }
    Shortcut { sequence: "Ctrl+Z";    onActivated: state.undo() }
    Shortcut { sequence: "Ctrl+Y";    onActivated: state.redo() }
    Shortcut { sequence: "L";         onActivated: state.toggle_loop() }
    Shortcut { sequence: "=";         onActivated: tl.zoom_in() }
    Shortcut { sequence: "-";         onActivated: tl.zoom_out() }
    Shortcut { sequence: "P";         onActivated: transport.toggle_pool() }
    Shortcut { sequence: "Ctrl+S";    onActivated: sc.save_project() }
    Shortcut { sequence: "Ctrl+O";    onActivated: sc.open_project() }
    Shortcut { sequence: "Ctrl+N";    onActivated: sc.new_project() }
    Shortcut { sequence: "Escape";    onActivated: sc.escape() }
    Shortcut { sequence: "Delete";    onActivated: sc.delete_selected() }
    Shortcut { sequence: "Ctrl+A";    onActivated: sc.select_all() }
    Shortcut { sequence: "Left";      onActivated: sc.nudge_left() }
    Shortcut { sequence: "Right";     onActivated: sc.nudge_right() }

    property bool poolVisible: transport.pool_visible
    property bool fxVisible: toolBar.fxVisible
    property bool mixerVisible: toolBar.mixerVisible

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        MenuBar { Layout.fillWidth: true }

        ToolBar {
            id: toolBar
            Layout.fillWidth: true
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: "#333333" }

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            PoolPanel {
                id: poolPanel
                Layout.fillHeight: true
                poolVisible: poolVisible
            }

            Rectangle { width: 1; Layout.fillHeight: true; color: "#333333"; visible: poolVisible }

            TimelineArea {
                id: timeline
                Layout.fillWidth: true
                Layout.fillHeight: true
                mixer: mixer
                stripModel: mixerPanel.stripModel
                fxVisible: fxVisible
                onFxToggleRequested: toolBar.fxVisible = !toolBar.fxVisible
            }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: "#333333" }

        RowLayout {
            Layout.fillWidth: true
            Layout.preferredHeight: fxVisible || mixerVisible ? 220 : 0
            spacing: 0

            FXEditor {
                Layout.fillHeight: true
                fxVisible: fxVisible
            }

            Rectangle { Layout.fillWidth: true; width: 1; height: 1; color: "#333333" }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: "#333333"; visible: mixerVisible }

        MixerPanel {
            id: mixerPanel
            Layout.fillWidth: true
            mixerVisible: mixerVisible
        }
    }
}
