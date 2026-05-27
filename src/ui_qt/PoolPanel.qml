import QtQuick 2.15
import QtQuick.Layouts 1.15
import com.hdaw 1.0

Rectangle {
    id: poolArea
    Layout.preferredWidth: poolVisible ? 280 : 0
    Layout.fillHeight: true
    visible: poolVisible
    clip: true
    color: "#121212"

    property bool poolVisible: false

    PoolModel { id: poolModel }

    Timer { interval: 300; running: true; repeat: true; onTriggered: poolModel.refresh() }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 24
            color: "#1a1a1a"
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 8
                anchors.rightMargin: 4
                Text {
                    text: "AUDIO POOL"
                    color: "#888888"
                    font.pixelSize: 11
                    verticalAlignment: Text.AlignVCenter
                }
                Item { Layout.fillWidth: true }
            }
        }

        ListView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            spacing: 1
            model: ListModel { id: poolListModel }

            delegate: Rectangle {
                width: parent.width
                height: 36
                color: mouseArea.containsMouse ? "#2a3a5a" : "transparent"
                radius: 3

                MouseArea {
                    id: mouseArea
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: poolModel.insert_pool_audio(model.path)
                }

                ColumnLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 4
                    anchors.rightMargin: 4
                    spacing: 0

                    Text {
                        text: model.name
                        color: "#cccccc"
                        font.pixelSize: 10
                        elide: Text.ElideRight
                    }

                    Text {
                        text: model.info
                        color: "#777777"
                        font.pixelSize: 8
                    }
                }
            }
        }
    }

    function updatePool(jsonStr) {
        var data
        try { data = JSON.parse(jsonStr) } catch (e) { return }
        if (!Array.isArray(data)) return
        poolListModel.clear()
        for (var i = 0; i < data.length; i++) {
            poolListModel.append({
                name: data[i].name,
                info: data[i].info,
                usage: data[i].usage,
                path: data[i].path
            })
        }
    }

    Connections {
        target: poolModel
        function onPool_jsonChanged() { poolArea.updatePool(poolModel.pool_json) }
    }

    Component.onCompleted: poolModel.refresh()
}
