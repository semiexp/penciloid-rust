use super::super::{Coord, X, Y};
use super::{Cell, Clue, CLUE_TYPES, CLUE_VALUES};

pub const DICTIONARY_NEIGHBOR_SIZE: usize = 8;
pub const DICTIONARY_NEIGHBOR_OFFSET: [Coord; DICTIONARY_NEIGHBOR_SIZE] = [
    (Y(-1), X(-1)),
    (Y(0), X(-1)),
    (Y(1), X(-1)),
    (Y(1), X(0)),
    (Y(1), X(1)),
    (Y(0), X(1)),
    (Y(-1), X(1)),
    (Y(-1), X(0)),
];
pub const DICTIONARY_INCONSISTENT: u32 = 0xffffffff;

const DICTIONARY_NEIGHBOR_PATTERN_COUNT: usize = 6561; // 3^8
const DICTIONARY_SIZE: usize = DICTIONARY_NEIGHBOR_PATTERN_COUNT * CLUE_TYPES;

pub struct Dictionary {
    dic: Vec<u32>,
}
impl Dictionary {
    pub fn complete() -> Dictionary {
        let mut dic = vec![0u32; DICTIONARY_SIZE];
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
                        let cand1 = dic[ofs + Dictionary::pattern_to_id(&pat)];
                        pat[p] = Cell::White;
                        let cand2 = dic[ofs + Dictionary::pattern_to_id(&pat)];

                        dic[ofs + pat_id] = cand1 & cand2;
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
                            dic[ofs + pat_id] = pat_id_bin;
                        } else {
                            dic[ofs + pat_id] = DICTIONARY_INCONSISTENT;
                        }
                    }
                }
            }
        }

        Dictionary { dic }
    }

    pub fn consult_raw(&self, c: Clue, neighbor_code: u32) -> u32 {
        let Clue(c) = c;
        self.dic[c as usize * DICTIONARY_NEIGHBOR_PATTERN_COUNT + neighbor_code as usize]
    }
    pub fn consult(&self, c: Clue, neighbor: &mut [Cell]) -> bool {
        let id = Dictionary::pattern_to_id(neighbor);
        let res = self.consult_raw(c, id as u32);

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
                        _ => if len > 0 {
                            ret.push(len);
                            len = 0;
                        },
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
            ret += pow * match pat[i] {
                Cell::Undecided => 0,
                Cell::Black => 1,
                Cell::White => 2,
            };
            pow *= 3;
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::clue_pattern_to_id;

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
        let complete_dic = Dictionary::complete();

        {
            let mut original = str_to_pattern("????????");
            let clue = clue_pattern_to_id(&[]).unwrap();
            let expected = str_to_pattern("........");
            complete_dic.consult(clue, &mut original);
            assert_eq!(original, expected);
        }
        {
            let mut original = str_to_pattern("????????");
            let clue = clue_pattern_to_id(&[8]).unwrap();
            let expected = str_to_pattern("########");
            complete_dic.consult(clue, &mut original);
            assert_eq!(original, expected);
        }
        {
            let mut original = str_to_pattern("..??..??");
            let clue = clue_pattern_to_id(&[1]).unwrap();
            let expected = str_to_pattern("..??..??");
            complete_dic.consult(clue, &mut original);
            assert_eq!(original, expected);
        }
        {
            let mut original = str_to_pattern("??.?????");
            let clue = clue_pattern_to_id(&[4]).unwrap();
            let expected = str_to_pattern("??.???#?");
            complete_dic.consult(clue, &mut original);
            assert_eq!(original, expected);
        }
        {
            let mut original = str_to_pattern("?.??????");
            let clue = clue_pattern_to_id(&[1, 1, 3]).unwrap();
            let expected = str_to_pattern("#.#?#?#?");
            complete_dic.consult(clue, &mut original);
            assert_eq!(original, expected);
        }
        {
            let mut original = str_to_pattern("?????.??");
            let clue = clue_pattern_to_id(&[2, 4]).unwrap();
            let expected = str_to_pattern("?#?##.##");
            complete_dic.consult(clue, &mut original);
            assert_eq!(original, expected);
        }
        {
            let mut original = str_to_pattern("?#??#???");
            let clue = clue_pattern_to_id(&[1, 1, 1, 1]).unwrap();
            assert_eq!(complete_dic.consult(clue, &mut original), true);
        }
    }
}
