import QtQuick 2.15
import QtQuick.Layouts 1.15

Rectangle {
    id: menuBar
    Layout.fillWidth: true
    Layout.preferredHeight: 28
    color: "#252525"

    property int openMenu: -1

    Row {
        anchors.fill: parent
        anchors.leftMargin: 4
        spacing: 0

        component MenuHeader: Rectangle {
            width: labelWidth; height: parent.height
            color: "transparent"
            property alias text: label.text
            property int labelWidth: 44
            Text {
                id: label
                anchors.centerIn: parent
                color: "#cccccc"
                font.pixelSize: 11
            }
        }

        MenuHeader { text: "File"; labelWidth: 44; MouseArea { anchors.fill: parent; onClicked: { menuBar.openMenu = (menuBar.openMenu === 0 ? -1 : 0) } } }
        MenuHeader { text: "Edit"; labelWidth: 40; MouseArea { anchors.fill: parent; onClicked: { menuBar.openMenu = (menuBar.openMenu === 1 ? -1 : 1) } } }
        MenuHeader { text: "Track"; labelWidth: 50; MouseArea { anchors.fill: parent; onClicked: { menuBar.openMenu = (menuBar.openMenu === 2 ? -1 : 2) } } }
        MenuHeader { text: "Transport"; labelWidth: 72; MouseArea { anchors.fill: parent; onClicked: { menuBar.openMenu = (menuBar.openMenu === 3 ? -1 : 3) } } }
        MenuHeader { text: "View"; labelWidth: 48; MouseArea { anchors.fill: parent; onClicked: { menuBar.openMenu = (menuBar.openMenu === 4 ? -1 : 4) } } }

        Item { width: parent.width - 258; height: 1 }
    }
}
