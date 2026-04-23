id: normal1
debug: [normal_diff, ]
let continentness = avg(fnl.conti, fnl.penin)
let continentness = *(continentness, normal_diff)
let continent_min = 0.3

out = idxmax(continent_min, continentness)



[] ocean
[] normal2
