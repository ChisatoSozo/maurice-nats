# automatically generated by the FlatBuffers compiler, do not modify

# namespace: 

import flatbuffers
from flatbuffers.compat import import_numpy
np = import_numpy()

class PlaylistEvent(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = PlaylistEvent()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsPlaylistEvent(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # PlaylistEvent
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # PlaylistEvent
    def DeviceId(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # PlaylistEvent
    def EventType(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Uint8Flags, o + self._tab.Pos)
        return 0

    # PlaylistEvent
    def Event(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(8))
        if o != 0:
            from flatbuffers.table import Table
            obj = Table(bytearray(), 0)
            self._tab.Union(obj, o)
            return obj
        return None

def PlaylistEventStart(builder):
    builder.StartObject(3)

def Start(builder):
    PlaylistEventStart(builder)

def PlaylistEventAddDeviceId(builder, deviceId):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(deviceId), 0)

def AddDeviceId(builder, deviceId):
    PlaylistEventAddDeviceId(builder, deviceId)

def PlaylistEventAddEventType(builder, eventType):
    builder.PrependUint8Slot(1, eventType, 0)

def AddEventType(builder, eventType):
    PlaylistEventAddEventType(builder, eventType)

def PlaylistEventAddEvent(builder, event):
    builder.PrependUOffsetTRelativeSlot(2, flatbuffers.number_types.UOffsetTFlags.py_type(event), 0)

def AddEvent(builder, event):
    PlaylistEventAddEvent(builder, event)

def PlaylistEventEnd(builder):
    return builder.EndObject()

def End(builder):
    return PlaylistEventEnd(builder)
