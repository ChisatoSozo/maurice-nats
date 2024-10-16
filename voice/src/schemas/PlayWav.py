# automatically generated by the FlatBuffers compiler, do not modify

# namespace: 

import flatbuffers
from flatbuffers.compat import import_numpy
np = import_numpy()

class PlayWav(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = PlayWav()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsPlayWav(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # PlayWav
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # PlayWav
    def Data(self, j):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            a = self._tab.Vector(o)
            return self._tab.Get(flatbuffers.number_types.Uint8Flags, a + flatbuffers.number_types.UOffsetTFlags.py_type(j * 1))
        return 0

    # PlayWav
    def DataAsNumpy(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.GetVectorAsNumpy(flatbuffers.number_types.Uint8Flags, o)
        return 0

    # PlayWav
    def DataLength(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.VectorLen(o)
        return 0

    # PlayWav
    def DataIsNone(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        return o == 0

def PlayWavStart(builder):
    builder.StartObject(1)

def Start(builder):
    PlayWavStart(builder)

def PlayWavAddData(builder, data):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(data), 0)

def AddData(builder, data):
    PlayWavAddData(builder, data)

def PlayWavStartDataVector(builder, numElems):
    return builder.StartVector(1, numElems, 1)

def StartDataVector(builder, numElems: int) -> int:
    return PlayWavStartDataVector(builder, numElems)

def PlayWavEnd(builder):
    return builder.EndObject()

def End(builder):
    return PlayWavEnd(builder)
