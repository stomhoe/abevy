id: desert1
size: (1, 1)
debug: [dune_feature]

let continentness = max(fnl.conti, fnl.penin)
let continentness = *(continentness, desert_diff)
let continent_threshold = 0.4

out = idxmax(continent_threshold, continentness)

bif "" -> [purple]
bif desert2 -> [] biomes: [desert=3.5(5.0,0.35)]
