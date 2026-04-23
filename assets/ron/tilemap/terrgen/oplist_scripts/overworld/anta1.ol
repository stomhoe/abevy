id: anta1
size: (1, 1)
debug: [anta_diff]

let continentness = avg(fnl.conti, fnl.penin)
let continentness = *(continentness, anta_diff)
let continent_min = 0.3

out = idxmax(continent_min, continentness)


[] ocean
[] biomes: [anta=3.5(5.0,0.35)] anta2
