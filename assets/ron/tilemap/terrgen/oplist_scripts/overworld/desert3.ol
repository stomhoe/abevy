id: desert3
tags: [portal, land, desert]
size: (1, 1)
debug: [oasisness]

let oasisness = *(fnl.lakebig, 0.37)
let oasisness = lerp(oasisness, inlandness, 0.01)

out = idxmax(continent_min, oasisness, )

[sand] desert4 biomes: [desert=1.0(30, 3)]
[saltwater] ""
