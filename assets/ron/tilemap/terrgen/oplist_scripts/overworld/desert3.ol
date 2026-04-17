id: desert3
tags: [portal, land, desert]
size: (1, 1)
debug: [oasis_feature]

let oasisness = *(fnl.lakesmol, 0.08)
let oasisness = lerp(oasisness, inlandness, 0.3)

out = idxmax(continent_threshold, oasisness, )

bif desert4 -> [sand] biomes: [desert=1.0(9.0,0.)]
bif "" -> [lake]
