id: anta1
size: (1, 1)
debug: [anta_diff]

let continentness = avg(fnl.conti, fnl.penin)
let continentness = *(continentness, anta_diff)
let continent_threshold = 0.3

out = idxmax(continent_threshold, continentness)


bif ocean -> []
bif anta2 -> [] biomes: [anta=3.5(5.0,0.35)]
