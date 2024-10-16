# automatically generated by the FlatBuffers compiler, do not modify

# namespace: 

import flatbuffers
from flatbuffers.compat import import_numpy
np = import_numpy()

class PlayStarted(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = PlayStarted()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsPlayStarted(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # PlayStarted
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # PlayStarted
    def ContentType(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Uint8Flags, o + self._tab.Pos)
        return 0

    # PlayStarted
    def Content(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            from flatbuffers.table import Table
            obj = Table(bytearray(), 0)
            self._tab.Union(obj, o)
            return obj
        return None

def PlayStartedStart(builder):
    builder.StartObject(2)

def Start(builder):
    PlayStartedStart(builder)

def PlayStartedAddContentType(builder, contentType):
    builder.PrependUint8Slot(0, contentType, 0)

def AddContentType(builder, contentType):
    PlayStartedAddContentType(builder, contentType)

def PlayStartedAddContent(builder, content):
    builder.PrependUOffsetTRelativeSlot(1, flatbuffers.number_types.UOffsetTFlags.py_type(content), 0)

def AddContent(builder, content):
    PlayStartedAddContent(builder, content)

def PlayStartedEnd(builder):
    return builder.EndObject()

def End(builder):
    return PlayStartedEnd(builder)
