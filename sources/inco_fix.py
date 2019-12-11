#MenuTitle: Fix Inconsolata for fontmake
__doc__="""
Decomposes components that have varying 2x2 matrices, and
also decomposes corners.
"""
# See https://github.com/googlefonts/fontmake/issues/595
for glyph in Glyphs.font.glyphs:
    #print glyph
    xforms = []
    mismatch = []
    for (i, layer) in enumerate(glyph.layers):
        for (j, component) in enumerate(layer.components):
            if i == 0:
                xforms.append(component.transform)
            else:
                if xforms[j][:4] != component.transform[:4]:
                    if j not in mismatch:
                        mismatch.append(j)
    if mismatch:
        print glyph.name, mismatch
        mismatch.reverse()
        for layer in glyph.layers:
            for j in mismatch:
                layer.components[j].decompose()

for glyph in Glyphs.font.glyphs:
    for layer in glyph.layers:
        has_corner = any(hint.type == 16 for hint in layer.hints)
        if has_corner:
            print glyph.name
            layer.decomposeCorners()
