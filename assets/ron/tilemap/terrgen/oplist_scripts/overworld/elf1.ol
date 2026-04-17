id: elf1

let continentness = avg(fnl.conti, fnl.penin)
let continentness = *(continentness, elf_diff)
let continent_threshold = 0.3
out = idxmax(continent_threshold, continentness)

bif ocean -> []
bif elf2 -> []
