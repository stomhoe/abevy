id: elf1

let continentness = avg(fnl.conti, fnl.penin)
let continentness = *(continentness, elf_diff)
let continent_min = 0.3
out = idxmax(continent_min, continentness)

[] ocean
[] elf2
