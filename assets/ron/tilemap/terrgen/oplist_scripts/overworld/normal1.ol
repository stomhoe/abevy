id: normal1
debug: [normal_diff, arcticriver]
let continentness = max(fnl.conti, fnl.penin)
let continentness = *(continentness, normal_diff)
let continent_threshold = 0.4
out = idxmax(continent_threshold, continentness)



bif ocean -> []
bif normal2 -> []
