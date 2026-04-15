id: arctic1
size: (1, 1)
debug: [boulder_feature]

let continentness = max(fnl.conti, fnl.penin)
let continentness = *(continentness, arctic_diff)
let continent_threshold = 0.4

out = idxmax(continent_threshold, continentness)


bif ocean -> []
bif arctic2 -> [] biomes: [arctic=3.5(5.0,0.35)]
