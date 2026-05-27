import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import com.hdaw 1.0

Rectangle {
    id: toolbar
    Layout.fillWidth: true
    Layout.preferredHeight: 32
    color: "#2a2a2a"

    TransportBar { id: transport }
    StateBridge { id: state }
    ShortcutHandler { id: sc }

    property bool fxVisible: false
    property bool mixerVisible: true
    readonly property bool poolVisible: transport.pool_visible

    Timer {
        interval: 16; running: true; repeat: true
        onTriggered: {
            transport.sync_state()
            state.sync_state()
            sc.sync_state()
        }
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 4
        anchors.rightMargin: 4
        spacing: 2

        Rectangle {
            Layout.preferredWidth: 28; Layout.preferredHeight: 24
            radius: 3; color: sc.tool_mode === 0 ? "#1a3a5c" : "transparent"
            Text { anchors.centerIn: parent; text: "S"; color: "#cccccc"; font.pixelSize: 11 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: sc.toggle_tool_mode() }
        }
        Rectangle {
            Layout.preferredWidth: 28; Layout.preferredHeight: 24
            radius: 3; color: sc.tool_mode !== 0 ? "#3a1a1a" : "transparent"
            Text { anchors.centerIn: parent; text: "C"; color: "#cccccc"; font.pixelSize: 11 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: sc.toggle_tool_mode() }
        }
        Rectangle {
            Layout.preferredWidth: 28; Layout.preferredHeight: 24
            radius: 3; color: sc.snap_enabled ? "#3a3a5c" : "transparent"
            Text { anchors.centerIn: parent; text: "M"; color: "#cccccc"; font.pixelSize: 11 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: sc.toggle_snap() }
        }
        Text { text: "|"; color: "#444444" }

        Rectangle {
            Layout.preferredWidth: 28; Layout.preferredHeight: 24; radius: 3
            color: "transparent"; opacity: state.undo_available ? 1.0 : 0.3
            Text { anchors.centerIn: parent; text: "\u21B6"; color: "#cccccc"; font.pixelSize: 13 }
            MouseArea {
                anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                onClicked: state.undo()
                enabled: state.undo_available
            }
        }
        Rectangle {
            Layout.preferredWidth: 28; Layout.preferredHeight: 24; radius: 3
            color: "transparent"; opacity: state.redo_available ? 1.0 : 0.3
            Text { anchors.centerIn: parent; text: "\u21B7"; color: "#cccccc"; font.pixelSize: 13 }
            MouseArea {
                anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                onClicked: state.redo()
                enabled: state.redo_available
            }
        }
        Text { text: "|"; color: "#444444" }

        Rectangle {
            Layout.preferredWidth: 28; Layout.preferredHeight: 24; radius: 3; color: "transparent"
            Text { anchors.centerIn: parent; text: "<<"; color: "#888888"; font.pixelSize: 10 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: sc.go_to_start() }
        }
        Rectangle {
            Layout.preferredWidth: 28; Layout.preferredHeight: 24; radius: 3; color: "transparent"
            Text { anchors.centerIn: parent; text: ">>"; color: "#888888"; font.pixelSize: 10 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: sc.go_to_end() }
        }
        Rectangle {
            Layout.preferredWidth: 40; Layout.preferredHeight: 24; radius: 3
            color: transport.playing ? "#1a5c1a" : "transparent"
            Text { anchors.centerIn: parent; text: transport.playing ? "Playing" : "Play"; color: transport.playing ? "#8bc34a" : "#cccccc"; font.pixelSize: 10 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: transport.play() }
        }
        Rectangle {
            Layout.preferredWidth: 36; Layout.preferredHeight: 24; radius: 3; color: "#4a1a1a"
            Text { anchors.centerIn: parent; text: "Stop"; color: "#f44336"; font.pixelSize: 10 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: transport.stop() }
        }
        Rectangle {
            Layout.preferredWidth: 32; Layout.preferredHeight: 24; radius: 3
            color: transport.recording ? "#5c1a1a" : "transparent"
            Text { anchors.centerIn: parent; text: "Rec"; color: transport.recording ? "#ff2222" : "#884444"; font.pixelSize: 10 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: transport.toggle_record() }
        }
        Rectangle {
            Layout.preferredWidth: 40; Layout.preferredHeight: 24; radius: 3
            color: state.loop_enabled ? "#3a3a5c" : "transparent"
            Text { anchors.centerIn: parent; text: "Loop"; color: state.loop_enabled ? "#64b5f6" : "#cccccc"; font.pixelSize: 10 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: state.toggle_loop() }
        }
        Text { text: "|"; color: "#444444" }

        Label {
            text: state.get_time_display()
            color: "#8bc34a"; font.pixelSize: 12; font.bold: true
            Layout.preferredWidth: 70; horizontalAlignment: Text.AlignHCenter
        }
        Label {
            text: state.get_bpm_display()
            color: "#cccccc"; font.pixelSize: 10
            Layout.preferredWidth: 80; horizontalAlignment: Text.AlignHCenter
        }
        Label {
            text: state.get_time_sig_display()
            color: "#cccccc"; font.pixelSize: 10
            Layout.preferredWidth: 40; horizontalAlignment: Text.AlignHCenter
        }
        Text { text: "|"; color: "#444444" }

        Item { Layout.fillWidth: true }

        Rectangle {
            Layout.preferredWidth: 55; Layout.preferredHeight: 24; radius: 3
            color: "#1a3a5c"
            Text { anchors.centerIn: parent; text: "Import"; color: "#64b5f6"; font.pixelSize: 10 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: transport.import_file() }
        }
        Rectangle {
            Layout.preferredWidth: 45; Layout.preferredHeight: 24; radius: 3
            color: poolVisible ? "#1a3a5c" : "transparent"
            Text { anchors.centerIn: parent; text: "Pool"; color: "#cccccc"; font.pixelSize: 10 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: transport.toggle_pool() }
        }
        Rectangle {
            Layout.preferredWidth: 35; Layout.preferredHeight: 24; radius: 3
            color: fxVisible ? "#3a5c3a" : "transparent"
            Text { anchors.centerIn: parent; text: "FX"; color: "#cccccc"; font.pixelSize: 10 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { fxVisible = !fxVisible } }
        }
        Rectangle {
            Layout.preferredWidth: 45; Layout.preferredHeight: 24; radius: 3
            color: mixerVisible ? "#3a3a1a" : "transparent"
            Text { anchors.centerIn: parent; text: "Mixer"; color: "#cccccc"; font.pixelSize: 10 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { mixerVisible = !mixerVisible } }
        }
    }
}
