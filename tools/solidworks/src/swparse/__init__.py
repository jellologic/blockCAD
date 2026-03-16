"""swparse – SolidWorks 3DEXPERIENCE file parser and toolkit.

Reverse-engineered binary format for SLDPRT / SLDASM files produced by the
SolidWorks 3DEXPERIENCE platform (NOT the legacy OLE2/Structured-Storage
format used by desktop SolidWorks before ~2020).

Quick start::

    from swparse import SldFile

    sld = SldFile.open("part.SLDPRT")
    for name in sld.stream_names():
        print(name, sld.streams[name].size)
"""

from .container import SldFile
from .header import FileHeader
from .records import Record
from .streams import ContentType, Stream

__all__ = ["SldFile", "FileHeader", "Record", "Stream", "ContentType"]
