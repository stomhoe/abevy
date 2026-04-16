id: desert3
tags: [portal, land, desert]
size: (1, 1)
debug: [oasis_feature]

let oasisness = *(max(fnl.lakesmol, fnl.lakebig), 0.5)
let oasisness = avg(oasisness, continentness)
let oasisness = *opo(oasisness, beachness)

out = idxmax(continent_threshold, oasisness, )

bif desert4 -> [sand] biomes: [desert=1.0(1.0,0.)]
bif "" -> [lake]
