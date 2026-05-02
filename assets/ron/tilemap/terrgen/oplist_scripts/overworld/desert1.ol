id: desert1
size: (1, 1)
debug: [dune_feature]

let continentness = avg(fnl.conti, fnl.penin)
let continentness = *(continentness, desert_diff)
let continent_min = 0.3

out = idxmax(continent_min, continentness)

[] ocean
[] desert2 biomes: [desert=3.5(5.0,0.35)]
