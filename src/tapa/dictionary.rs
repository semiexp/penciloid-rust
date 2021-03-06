use super::super::D;
use super::{Cell, Clue, CLUE_TYPES, CLUE_VALUES};

pub const DICTIONARY_NEIGHBOR_SIZE: usize = 8;
pub const DICTIONARY_NEIGHBOR_OFFSET: [D; DICTIONARY_NEIGHBOR_SIZE] = [
    D(-1, -1),
    D(0, -1),
    D(1, -1),
    D(1, 0),
    D(1, 1),
    D(0, 1),
    D(-1, 1),
    D(-1, 0),
];
pub const DICTIONARY_INCONSISTENT: u32 = 0xffffffff;

// 0: undecided, 1: black, 2: white
const DICTIONARY_NEIGHBOR_PATTERN_COUNT: usize = 6561; // 3^8
const DICTIONARY_NEIGHBOR_PATTERN_TOTAL_SIZE: usize =
    DICTIONARY_NEIGHBOR_PATTERN_COUNT * CLUE_TYPES;

// 0: undecided, 1: degree=2, 2: black, 3: white
const DICTIONARY_VIRTUALLY_IGNORED_CELL_COUNT: usize = 1 << 16; // 4^8 = 2^16
const DICTIONARY_VIRTUALLY_IGNORED_TOTAL_SIZE: usize =
    DICTIONARY_VIRTUALLY_IGNORED_CELL_COUNT * CLUE_TYPES;

pub struct Dictionary {
    neighbor_pattern: Vec<u32>,
    virtually_ignored_cell: Vec<u32>,
    virtual_disconnection: Vec<u32>,
}
impl Dictionary {
    pub fn new() -> Dictionary {
        let mut neighbor_pattern = vec![0u32; DICTIONARY_NEIGHBOR_PATTERN_TOTAL_SIZE];
        for ty in 0..CLUE_TYPES {
            let ofs = ty * DICTIONARY_NEIGHBOR_PATTERN_COUNT;
            for pat_id in 0..DICTIONARY_NEIGHBOR_PATTERN_COUNT {
                let pat_id = DICTIONARY_NEIGHBOR_PATTERN_COUNT - 1 - pat_id;
                let mut pat = Dictionary::id_to_pattern(pat_id);

                let mut undecided_loc = None;
                for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                    if pat[i] == Cell::Undecided {
                        undecided_loc = Some(i);
                        break;
                    }
                }

                match undecided_loc {
                    Some(p) => {
                        pat[p] = Cell::Black;
                        let cand1 = neighbor_pattern[ofs + Dictionary::pattern_to_id(&pat)];
                        pat[p] = Cell::White;
                        let cand2 = neighbor_pattern[ofs + Dictionary::pattern_to_id(&pat)];

                        neighbor_pattern[ofs + pat_id] = cand1 & cand2;
                    }
                    None => {
                        let chains = Dictionary::neighbor_chain(&pat);
                        if (&chains)
                            .into_iter()
                            .eq(CLUE_VALUES[ty].into_iter().filter(|&&x| x != -1))
                        {
                            let mut pat_id_bin = 0u32;
                            for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                                pat_id_bin |= match pat[i] {
                                    Cell::Black => 1,
                                    Cell::White => 2,
                                    _ => unreachable!(),
                                } << (2 * i);
                            }
                            neighbor_pattern[ofs + pat_id] = pat_id_bin;
                        } else {
                            neighbor_pattern[ofs + pat_id] = DICTIONARY_INCONSISTENT;
                        }
                    }
                }
            }
        }

        let mut virtual_disconnection = vec![0u32; DICTIONARY_NEIGHBOR_PATTERN_TOTAL_SIZE];
        for ty in 0..CLUE_TYPES {
            let ofs = ty * DICTIONARY_NEIGHBOR_PATTERN_COUNT;
            for pat_id in 0..DICTIONARY_NEIGHBOR_PATTERN_COUNT {
                let mut disconnection = 0u32;
                let pat = Dictionary::id_to_pattern(pat_id);
                for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                    if pat[i] == Cell::White
                        || pat[(i + 1) % DICTIONARY_NEIGHBOR_SIZE] == Cell::White
                    {
                        continue;
                    }
                    let mut pat = pat.clone();
                    pat[i] = Cell::Black;
                    pat[(i + 1) % DICTIONARY_NEIGHBOR_SIZE] = Cell::Black;

                    if neighbor_pattern[ofs + Dictionary::pattern_to_id(&pat)]
                        == DICTIONARY_INCONSISTENT
                    {
                        disconnection |= 1 << i;
                    }
                }
                virtual_disconnection[ofs + pat_id] = disconnection;
            }
        }

        let mut virtually_ignored_cell = vec![0u32; DICTIONARY_VIRTUALLY_IGNORED_TOTAL_SIZE];
        for ty in 0..CLUE_TYPES {
            let ofs = ty * DICTIONARY_VIRTUALLY_IGNORED_CELL_COUNT;
            for pat_id in 0..DICTIONARY_VIRTUALLY_IGNORED_CELL_COUNT {
                let mut ori = None;
                for i in 0..8 {
                    if ((pat_id >> (2 * i)) & 3) != 1 {
                        ori = Some(i);
                        break;
                    }
                }

                if let Some(ori) = ori {
                    let mut chain_length = 0;
                    let mut ignored_cells_mask = 0u32;
                    for i in 0..9 {
                        let i = (i + ori) % 8;

                        if ((pat_id >> (2 * i)) & 3) != 1 {
                            if chain_length > 0 {
                                let mut pat_id_blackened = pat_id;
                                for j in 0..(chain_length + 2) {
                                    let j = (i - j + 8) % 8;
                                    pat_id_blackened =
                                        (pat_id_blackened & !(3 << (2 * j))) | (2 << (2 * j));
                                }

                                let mut ternary_pat_id = 0;
                                let mut pow3 = 1;
                                for j in 0..8 {
                                    let p = (pat_id_blackened >> (2 * j)) & 3;
                                    let p = if p == 0 { 0 } else { p - 1 };
                                    ternary_pat_id += p * pow3;
                                    pow3 *= 3;
                                }

                                if neighbor_pattern
                                    [ty * DICTIONARY_NEIGHBOR_PATTERN_COUNT + ternary_pat_id]
                                    == DICTIONARY_INCONSISTENT
                                {
                                    for j in 0..chain_length {
                                        let j = (i - j + 7) % 8;
                                        ignored_cells_mask |= 1 << j;
                                    }
                                }

                                chain_length = 0;
                            }
                        } else {
                            chain_length += 1;
                        }
                    }

                    virtually_ignored_cell[ofs + pat_id] = ignored_cells_mask;
                }
            }
        }

        Dictionary {
            neighbor_pattern,
            virtually_ignored_cell,
            virtual_disconnection,
        }
    }

    pub fn neighbor_pattern_raw(&self, c: Clue, neighbor_code: u32) -> u32 {
        let Clue(c) = c;
        self.neighbor_pattern
            [c as usize * DICTIONARY_NEIGHBOR_PATTERN_COUNT + neighbor_code as usize]
    }
    pub fn neighbor_pattern(&self, c: Clue, neighbor: &mut [Cell]) -> bool {
        let id = Dictionary::pattern_to_id(neighbor);
        let res = self.neighbor_pattern_raw(c, id as u32);

        if res == DICTIONARY_INCONSISTENT {
            true
        } else {
            for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                if neighbor[i] == Cell::Undecided {
                    neighbor[i] = match (res >> (2 * i)) & 3 {
                        1 => Cell::Black,
                        2 => Cell::White,
                        _ => Cell::Undecided,
                    }
                }
            }
            false
        }
    }

    pub fn virtually_ignored_cell(&self, c: Clue, neighbor_code: u32) -> u32 {
        let Clue(c) = c;
        self.virtually_ignored_cell
            [c as usize * DICTIONARY_VIRTUALLY_IGNORED_CELL_COUNT + neighbor_code as usize]
    }
    pub fn virtual_disconnection(&self, c: Clue, neighbor_code: u32) -> u32 {
        let Clue(c) = c;
        self.virtual_disconnection
            [c as usize * DICTIONARY_NEIGHBOR_PATTERN_COUNT + neighbor_code as usize]
    }

    fn neighbor_chain(pattern: &[Cell]) -> Vec<i32> {
        let mut top = None;
        for i in 0..DICTIONARY_NEIGHBOR_SIZE {
            if pattern[i] != Cell::Black {
                top = Some(i);
                break;
            }
        }
        match top {
            Some(top) => {
                let mut ret = vec![];
                let mut len = 0;
                for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                    let i = (i + top) % DICTIONARY_NEIGHBOR_SIZE;
                    match pattern[i] {
                        Cell::Black => len += 1,
                        _ => {
                            if len > 0 {
                                ret.push(len);
                                len = 0;
                            }
                        }
                    }
                }
                if len > 0 {
                    ret.push(len);
                }
                ret.sort();
                ret
            }
            None => vec![DICTIONARY_NEIGHBOR_SIZE as i32],
        }
    }
    fn id_to_pattern(pat_id: usize) -> [Cell; DICTIONARY_NEIGHBOR_SIZE] {
        let mut ret = [Cell::Undecided; DICTIONARY_NEIGHBOR_SIZE];
        let mut v = pat_id as u32;
        for i in 0..DICTIONARY_NEIGHBOR_SIZE {
            ret[i] = match v % 3 {
                0 => Cell::Undecided,
                1 => Cell::Black,
                2 => Cell::White,
                _ => unreachable!(),
            };
            v /= 3;
        }
        ret
    }
    fn pattern_to_id(pat: &[Cell]) -> usize {
        let mut ret = 0usize;
        let mut pow = 1usize;
        for i in 0..DICTIONARY_NEIGHBOR_SIZE {
            ret += pow
                * match pat[i] {
                    Cell::Undecided => 0,
                    Cell::Black => 1,
                    Cell::White => 2,
                };
            pow *= 3;
        }
        ret
    }
}

const CONSECUTIVE_DICTIONARY_NEIGHBOR_PATTERN_COUNT: usize = 256; // 2^8
const CONSECUTIVE_DICTIONARY_SIZE: usize =
    CONSECUTIVE_DICTIONARY_NEIGHBOR_PATTERN_COUNT * CLUE_TYPES;
pub const CONSECUTIVE_DICTIONARY_ADJACENCY_SIZE: usize = 4;
pub const CONSECUTIVE_DICTIONARY_ADJACENCY_OFFSET: [D; CONSECUTIVE_DICTIONARY_ADJACENCY_SIZE] =
    [D(-1, 0), D(0, -1), D(1, 0), D(0, 1)];
const CONSECUTIVE_DICTIONARY_REMOVAL_SIZE: usize = CONSECUTIVE_DICTIONARY_NEIGHBOR_PATTERN_COUNT
    * DICTIONARY_NEIGHBOR_SIZE
    * CONSECUTIVE_DICTIONARY_ADJACENCY_SIZE;
pub struct ConsecutiveRegionDictionary {
    dic: Vec<u32>,
    removal_dic: Vec<i32>,
    chain_size_dic: Vec<i32>,
}
impl ConsecutiveRegionDictionary {
    pub fn new(dic_base: &Dictionary) -> ConsecutiveRegionDictionary {
        let mut dic = vec![0u32; CONSECUTIVE_DICTIONARY_SIZE];
        for ty in 0..CLUE_TYPES {
            for pat_id in 0..CONSECUTIVE_DICTIONARY_NEIGHBOR_PATTERN_COUNT {
                let mut top = 0;
                for i in 0..8 {
                    if ((pat_id >> i) & 1) == 0 {
                        top = i;
                        break;
                    }
                }
                let mut chains = vec![];
                let mut chain_length = 0;
                for i in 0..8 {
                    let i = (i + top) % 8;
                    if ((pat_id >> i) & 1) == 1 {
                        chain_length += 1;
                    } else {
                        if chain_length != 0 {
                            chains.push(((i + 8 - chain_length) % 8, chain_length));
                            chain_length = 0;
                        }
                    }
                }
                if chain_length != 0 {
                    chains.push(((top + 8 - chain_length) % 8, chain_length));
                    chain_length = 0;
                }

                let mut parts = 0u32;

                for i in 0..chains.len() {
                    let mut pat_id_after_exclusion = pat_id;
                    let (p, len) = chains[i];
                    for j in 0..len {
                        pat_id_after_exclusion ^= 1 << ((p + j) % 8);
                    }

                    let mut ternary_id = 0;
                    let mut pow3 = 1;
                    for j in 0..8 {
                        if (pat_id_after_exclusion >> j) & 1 == 0 {
                            ternary_id += pow3 * 2;
                        }
                        pow3 *= 3;
                    }

                    if dic_base.neighbor_pattern_raw(Clue(ty as i32), ternary_id)
                        == DICTIONARY_INCONSISTENT
                    {
                        for j in 0..len {
                            parts |= 1 << (((p + j) % 8) as u32);
                        }
                    }
                }

                dic[ty * CONSECUTIVE_DICTIONARY_NEIGHBOR_PATTERN_COUNT + pat_id] = parts;
            }
        }

        let mut prev_direction = [0usize; DICTIONARY_NEIGHBOR_SIZE];
        let mut next_direction = [0usize; DICTIONARY_NEIGHBOR_SIZE];
        for i in 0..DICTIONARY_NEIGHBOR_SIZE {
            let ofs = DICTIONARY_NEIGHBOR_OFFSET[i];
            let ofs_prev = DICTIONARY_NEIGHBOR_OFFSET[(i + 7) % 8];
            let ofs_next = DICTIONARY_NEIGHBOR_OFFSET[(i + 1) % 8];
            for j in 0..4 {
                let d = CONSECUTIVE_DICTIONARY_ADJACENCY_OFFSET[j];
                if ofs_prev == ofs + d {
                    prev_direction[i] = j;
                }
                if ofs_next == ofs + d {
                    next_direction[i] = j;
                }
            }
        }

        let mut removal_dic = vec![0i32; CONSECUTIVE_DICTIONARY_REMOVAL_SIZE];
        let mut chain_size_dic =
            vec![0i32; CONSECUTIVE_DICTIONARY_NEIGHBOR_PATTERN_COUNT * DICTIONARY_NEIGHBOR_SIZE];
        for pat_id in 0..CONSECUTIVE_DICTIONARY_NEIGHBOR_PATTERN_COUNT {
            for nb in 0..DICTIONARY_NEIGHBOR_SIZE {
                if (pat_id >> nb) & 1 == 0 {
                    continue;
                }
                let mut prev_count = 0;
                let mut next_count = 0;
                if pat_id == CONSECUTIVE_DICTIONARY_NEIGHBOR_PATTERN_COUNT - 1 {
                    prev_count = 7;
                } else {
                    for i in 1..8 {
                        if (pat_id >> ((nb + 8 - i) % 8)) & 1 != 0 {
                            prev_count += 1;
                        } else {
                            break;
                        }
                    }
                    for i in 1..8 {
                        if (pat_id >> ((nb + i) % 8)) & 1 != 0 {
                            next_count += 1;
                        } else {
                            break;
                        }
                    }
                }
                chain_size_dic[pat_id * DICTIONARY_NEIGHBOR_SIZE + nb] =
                    prev_count + 1 + next_count;

                if prev_count > 0 && next_count > 0 {
                    removal_dic[(pat_id * DICTIONARY_NEIGHBOR_SIZE + nb)
                        * CONSECUTIVE_DICTIONARY_ADJACENCY_SIZE
                        + prev_direction[nb]] = prev_count;
                    removal_dic[(pat_id * DICTIONARY_NEIGHBOR_SIZE + nb)
                        * CONSECUTIVE_DICTIONARY_ADJACENCY_SIZE
                        + next_direction[nb]] = next_count;
                }
            }
        }

        ConsecutiveRegionDictionary {
            dic,
            removal_dic,
            chain_size_dic,
        }
    }
    pub fn consult(&self, c: Clue, neighbor_code: u32) -> u32 {
        let Clue(c) = c;
        self.dic
            [c as usize * CONSECUTIVE_DICTIONARY_NEIGHBOR_PATTERN_COUNT + neighbor_code as usize]
    }
    pub fn consult_removal(&self, code: u32, neighbor_id: usize, adjacency_id: usize) -> i32 {
        self.removal_dic[(code as usize * DICTIONARY_NEIGHBOR_SIZE + neighbor_id)
            * CONSECUTIVE_DICTIONARY_ADJACENCY_SIZE
            + adjacency_id]
    }
    pub fn chain_size(&self, code: u32, neighbor_id: usize) -> i32 {
        self.chain_size_dic[code as usize * DICTIONARY_NEIGHBOR_SIZE + neighbor_id]
    }
}

#[cfg(test)]
mod tests {
    use super::super::clue_pattern_to_id;
    use super::*;

    fn str_to_pattern(s: &str) -> [Cell; 8] {
        let mut ret = [Cell::Undecided; 8];
        let mut it = s.chars();
        for i in 0..8 {
            let top = it.next().unwrap();
            ret[i] = match top {
                '?' => Cell::Undecided,
                '#' => Cell::Black,
                '.' => Cell::White,
                _ => panic!("unexpected character"),
            };
        }
        ret
    }
    #[test]
    fn test_tapa_dictionary() {
        let complete_dic = Dictionary::new();

        {
            let mut original = str_to_pattern("????????");
            let clue = clue_pattern_to_id(&[]).unwrap();
            let expected = str_to_pattern("........");
            complete_dic.neighbor_pattern(clue, &mut original);
            assert_eq!(original, expected);
        }
        {
            let mut original = str_to_pattern("????????");
            let clue = clue_pattern_to_id(&[8]).unwrap();
            let expected = str_to_pattern("########");
            complete_dic.neighbor_pattern(clue, &mut original);
            assert_eq!(original, expected);
        }
        {
            let mut original = str_to_pattern("..??..??");
            let clue = clue_pattern_to_id(&[1]).unwrap();
            let expected = str_to_pattern("..??..??");
            complete_dic.neighbor_pattern(clue, &mut original);
            assert_eq!(original, expected);
        }
        {
            let mut original = str_to_pattern("??.?????");
            let clue = clue_pattern_to_id(&[4]).unwrap();
            let expected = str_to_pattern("??.???#?");
            complete_dic.neighbor_pattern(clue, &mut original);
            assert_eq!(original, expected);
        }
        {
            let mut original = str_to_pattern("?.??????");
            let clue = clue_pattern_to_id(&[1, 1, 3]).unwrap();
            let expected = str_to_pattern("#.#?#?#?");
            complete_dic.neighbor_pattern(clue, &mut original);
            assert_eq!(original, expected);
        }
        {
            let mut original = str_to_pattern("?????.??");
            let clue = clue_pattern_to_id(&[2, 4]).unwrap();
            let expected = str_to_pattern("?#?##.##");
            complete_dic.neighbor_pattern(clue, &mut original);
            assert_eq!(original, expected);
        }
        {
            let mut original = str_to_pattern("?#??#???");
            let clue = clue_pattern_to_id(&[1, 1, 1, 1]).unwrap();
            assert_eq!(complete_dic.neighbor_pattern(clue, &mut original), true);
        }
    }

    #[test]
    fn test_tapa_consecutive_dictionary() {
        let dic_base = Dictionary::new();
        let dic = ConsecutiveRegionDictionary::new(&dic_base);

        {
            let original = 0b11111111;
            let clue = clue_pattern_to_id(&[]).unwrap();
            let expected = 0b00000000;
            assert_eq!(dic.consult(clue, original), expected);
        }
        {
            let original = 0b11111111;
            let clue = clue_pattern_to_id(&[1]).unwrap();
            let expected = 0b11111111;
            assert_eq!(dic.consult(clue, original), expected);
        }
        {
            let original = 0b11101111;
            let clue = clue_pattern_to_id(&[3]).unwrap();
            let expected = 0b11101111;
            assert_eq!(dic.consult(clue, original), expected);
        }
        {
            let original = 0b11101101;
            let clue = clue_pattern_to_id(&[1, 2]).unwrap();
            let expected = 0b11100001;
            assert_eq!(dic.consult(clue, original), expected);
        }
    }
}
