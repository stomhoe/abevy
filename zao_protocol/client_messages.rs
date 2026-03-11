use bevy_replicon::prelude::Channel;
use serde::{Deserialize, Serialize};

use super::channels::{AO_MOVEMENT_CORRECTION_CHANNEL, Channel::Unordered};
use super::types::AoHeading;

/*
Client VB6 snippets from Argentum Online 20:
Public Sub WriteWalk(ByVal Heading As E_Heading)
    Call Writer.WriteInt16(ClientPacketID.eWalk)
    Call Writer.WriteInt8(Heading)
    packetCounters.TS_Walk = packetCounters.TS_Walk + 1
    Call Writer.WriteInt32(packetCounters.TS_Walk)
    Call modNetwork.send(Writer)
End Sub

Public Sub WriteRequestPositionUpdate()
    Call Writer.WriteInt16(ClientPacketID.eRequestPositionUpdate)
    Call modNetwork.send(Writer)
End Sub

Public Sub WriteChangeHeading(ByVal Heading As E_Heading)
    Call Writer.WriteInt16(ClientPacketID.eChangeHeading)
    Call Writer.WriteInt8(Heading)
    packetCounters.TS_ChangeHeading = packetCounters.TS_ChangeHeading + 1
    Call Writer.WriteInt32(packetCounters.TS_ChangeHeading)
    Call modNetwork.send(Writer)
End Sub

Server VB6 handlers:
Private Sub HandleWalk(ByVal UserIndex As Integer)
    Heading = reader.ReadInt8()
    PacketCount = reader.ReadInt32
    Call verifyTimeStamp(...)
    If MoveUserChar(UserIndex, Heading) Then
        Call ResetUserAutomatedActions(UserIndex)
        Call WritePosUpdate(UserIndex)
    Else
        Call WritePosUpdate(UserIndex)
    End If
End Sub

Private Sub HandleChange_Heading(ByVal UserIndex As Integer)
    Heading = reader.ReadInt8()
    PacketCounter = reader.ReadInt32
    If verifyTimeStamp(...) Then
        If Heading > 0 And Heading < 5 Then
            .Char.Heading = Heading
            Call SendData(..., PrepareMessageCharacterChange(...))
        End If
    End If
End Sub

Private Sub HandleRequestPositionUpdate(ByVal UserIndex As Integer)
    Call WritePosUpdate(UserIndex)
End Sub
*/

#[derive(Debug, Clone, Copy, Message, Serialize, Deserialize)]
pub struct AoWalkInput {
    pub heading: AoHeading,
    pub sequence: u32,
}

impl AoWalkInput {
    pub const CHANNEL: Channel = Channel::Unordered;
}

#[derive(Debug, Clone, Copy, Message, Serialize, Deserialize)]
pub struct AoChangeHeadingInput {
    pub heading: AoHeading,
    pub sequence: u32,
}

impl AoChangeHeadingInput {
    pub const CHANNEL: Channel = Channel::Unordered;
}

#[derive(Debug, Clone, Copy, Message, Serialize, Deserialize, Default)]
pub struct AoRequestPositionSync;

impl AoRequestPositionSync {
    pub const CHANNEL: Channel = Channel::Unordered;
}
