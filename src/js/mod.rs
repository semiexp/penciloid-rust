use super::*;
use rand::SeedableRng;

static mut SHARED_ARRAY: [u8; 1 << 16] = [0; 1 << 16];

#[no_mangle]
pub extern "C" fn numberlink_generate(height: i32, width: i32, seed1: f64, seed2: f64) -> *const u8 {
    let seed_array: [u8; 16] = unsafe { std::mem::transmute_copy(&(seed1, seed2)) };
    let mut rng = rand::prng::XorShiftRng::from_seed(seed_array);
    let mut generator = numberlink::PlacementGenerator::new(height, width);
    let cond = numberlink::GeneratorOption {
        chain_threshold: 8,
        endpoint_constraint: None,
        forbid_adjacent_clue: true,
        symmetry: Symmetry::none(),
        clue_limit: None,
        prioritized_extension: false,
    };
    loop {
        let generated = generator.generate(&cond, &mut rng);
        if let Some(line_placement) = generated {
            if !numberlink::uniqueness_pretest(&line_placement) { continue; }

            let problem = numberlink::extract_problem(&line_placement, &mut rng);

            let ans = numberlink::solve2(&problem, Some(2), false, true);

            if ans.len() == 1 && !ans.found_not_fully_filled {
                unsafe {
                    for y in 0..height {
                        for x in 0..width {
                            let val = problem[(Y(y), X(x))];
                            SHARED_ARRAY[(y * width + x) as usize] = val.0 as u8;
                        }
                    }
                    return &SHARED_ARRAY[0];
                }
            }
        }
    }
}
