id: normal1
debug: [normal_diff, ]
let continentness = avg(fnl.conti, fnl.penin)
let continentness = *(continentness, normal_diff)
let continent_threshold = 0.3
out = idxmax(continent_threshold, continentness)



bif "" -> [ocean]
bif normal2 -> []
