id: desert3
tags: [portal, land, desert]
size: (1, 1)
debug: [oasis_feature]

let oasisness = max(fnl.lakesmol, fnl.lakebig)
let oasisness = *(oasisness, continentness, 1.4)
let oasisness = *opo(oasisness, dune_sandness)

out = idxmax(continent_threshold, oasisness, )

bif desert4 -> [sand]
bif "" -> [lake]
bif "" -> [sand] biomes: [desert=1.0(1.0,0.)]
