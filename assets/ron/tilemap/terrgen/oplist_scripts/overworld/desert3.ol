id: desert3
tags: [portal, land, desert]
size: (1, 1)
debug: [oasisness]

let oasisness = *(fnl.lakesmol, 0.02)
let oasisness = lerp(oasisness, inlandness, 0.1)

out = idxmax(continent_min, oasisness, )

[sand] desert4 biomes: [desert=1.0(30, 3)]
[lake] ""
