# automatically generated by the FlatBuffers compiler, do not modify

# namespace: 

import flatbuffers
from flatbuffers.compat import import_numpy
np = import_numpy()

class Song(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = Song()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsSong(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # Song
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # Song
    def Url(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # Song
    def ThumbnailB64(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # Song
    def Title(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(8))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

def SongStart(builder):
    builder.StartObject(3)

def Start(builder):
    SongStart(builder)

def SongAddUrl(builder, url):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(url), 0)

def AddUrl(builder, url):
    SongAddUrl(builder, url)

def SongAddThumbnailB64(builder, thumbnailB64):
    builder.PrependUOffsetTRelativeSlot(1, flatbuffers.number_types.UOffsetTFlags.py_type(thumbnailB64), 0)

def AddThumbnailB64(builder, thumbnailB64):
    SongAddThumbnailB64(builder, thumbnailB64)

def SongAddTitle(builder, title):
    builder.PrependUOffsetTRelativeSlot(2, flatbuffers.number_types.UOffsetTFlags.py_type(title), 0)

def AddTitle(builder, title):
    SongAddTitle(builder, title)

def SongEnd(builder):
    return builder.EndObject()

def End(builder):
    return SongEnd(builder)
