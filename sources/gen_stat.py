"""
Generates a .stylespace file for statmake
https://github.com/daltonmaag/statmake

Since Inconsolata has both a weight and width axis, the STAT table needs
Format 4 locations. This bumps the STAT table version from v1.1 to v1.2.
AFAIK v1.2 isn't supported by Microsoft yet.
"""
import plistlib
from fontTools.ttLib import TTFont


TAG_TO_NAME = {
    "wght": "Weight",
    "wdth": "Width",
}

font = TTFont('../fonts/variable/Inconsolata[wdth,wght].ttf')

fvar = font['fvar']
fvar_axes = fvar.axes
fvar_instances = fvar.instances

axes = [
    {"tag": a.axisTag,
     "name": font['name'].getName(a.axisNameID, 3, 1, 1033).toUnicode()}
     for a in fvar_axes
]

locations = [
    {"name": font['name'].getName(i.subfamilyNameID, 3, 1, 1033).toUnicode(),
     "axis_values": {TAG_TO_NAME[k]: v for k,v in i.coordinates.items()}}
     for i in fvar_instances
]

stylespace = dict(axes=axes, locations=locations)
with open("stat.stylespace", "wb") as doc:
    plistlib.dump(stylespace, doc)

print('Finished generating stylespace')
