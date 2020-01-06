#MenuTitle: Generate additional Inconsolata instances
__doc__="""
Generates additional interpolation instances for Inconsolata.
"""
widths = [
    ("Ultra Condensed", 50),
    ("Extra Condensed", 70),
    ("Condensed", 80),
    ("Semi Condensed", 90),
    ("Medium", 100),
    ("Semi Expanded", 110),
    ("Expanded", 120),
    ("Extra Expanded", 150),
    ("Ultra Expanded", 200),
]

weights = [
    ("ExtraLight", 200),
    ("Light", 300),
    ("Regular", 400),
    ("Medium", 500),
    ("SemiBold", 600),
    ("Bold", 700),
    ("ExtraBold", 800),
    ("Black", 900),
]

for wdth in widths:
    for wght in weights:
        if wdth[1] != 100 and not (wght[1] in (100, 400, 900) and wdth[1] in (50, 100, 200)):
            instance = GSInstance()
            instance.width = wdth[0]
            instance.weight = wght[0]
            name = wdth[0].replace(" ", "")
            if wght[0] != "Regular":
                name += " " + wght[0]
            instance.weightValue = wght[1]
            instance.widthValue = wdth[1]
            instance.name = name
            Glyphs.font.instances.append(instance)
