use std::collections::{HashMap, HashSet};

use glifparser::contour::State;

use crate::viewer::UFO;

fn create_glyph_set(masters: &Vec<UFO>) -> HashSet<String> {
    let mut glyph_set = HashSet::new();

    for ufo in masters {
        for entry in &ufo.glyph_entries {
            glyph_set.insert(entry.uniname.clone());
        }
    }

    return glyph_set;
}

// returns the names of missing glyphs in a given master as compared to the glyph set
// created by the glyphs of all masters
fn get_master_glyph_set_difference(master: &UFO, glyph_set: &HashSet<String>) -> HashSet<String> {
    let master_set: HashSet<String> = master.glyph_entries
        .iter()
        .map(|e| e.uniname.clone())
        .collect();

    let difference: HashSet<_> = glyph_set.difference(&master_set).cloned().collect();

    difference
}

pub struct InterpolationCheckResults {
    pub succeeded: bool,
    pub glyph_set: HashSet<String>,
    pub contour_count_collisions: HashSet<String>,
    pub point_count_collisions: HashSet<String>,
    pub contour_open_collisions: HashSet<String>,
    pub combined: HashSet<String>,
}

pub(crate) fn check_interpolatable(masters: &Vec<UFO>) -> InterpolationCheckResults {
    // create the set of all glyphs from all masters
    let glyph_set = create_glyph_set(masters);

    // get the missing glyphs from each master
    let glyph_set_differences: Vec<HashSet<String>> = masters
        .iter()
        .map(|m| get_master_glyph_set_difference(m, &glyph_set))
        .collect();

    // Combine all sets into one
    let combined_differences: HashSet<String> = glyph_set_differences
        .into_iter()
        .flatten()
        .collect();

    let contour_counts = count_different_contour_counts(masters);
    let contour_counts_set: HashSet<String> = contour_counts.into_iter().map(|(s, _)| s).collect();

    let point_counts = count_different_point_counts(masters);
    let point_counts_set: HashSet<String> = point_counts.into_iter().map(|(s, _)| s).collect();

    let open_states_counts = count_different_contour_open_state(masters);
    let open_states_counts_set: HashSet<String> = open_states_counts.into_iter().map(|(s, _)| s).collect();

    let mut success = false;
    if combined_differences.is_empty() && contour_counts_set.is_empty() && point_counts_set.is_empty() && open_states_counts_set.is_empty() {
        success = true;
    }

    let combined_sets: HashSet<String> = combined_differences.union(&point_counts_set).cloned().collect();
    let combined_sets: HashSet<String> = combined_sets.union(&contour_counts_set).cloned().collect();
    let combined_sets: HashSet<String> = combined_sets.union(&open_states_counts_set).cloned().collect();

    return InterpolationCheckResults {
        succeeded: success,
        glyph_set,
        contour_count_collisions: contour_counts_set,
        point_count_collisions: point_counts_set,
        contour_open_collisions: open_states_counts_set,
        combined: combined_sets
    }
}

fn count_different_contour_counts(masters: &Vec<UFO>) -> HashMap<String, usize> {
    let dummy = Vec::new();
    // for each master grab a contour count
    let contour_count_set: Vec<HashMap<String, usize>> = masters
        .iter()
        .map(|v| {
            v.glyph_entries
                .iter()
                .map(|e| {
                    (e.uniname.clone(), e.glif.outline.as_ref().unwrap_or(&dummy).len())
                })
                .collect()
        })
        .collect();
    
    let mut counts: HashMap<String, HashSet<usize>> = HashMap::new();

    for count_set in contour_count_set {
        for (idx, count) in count_set {
            counts.entry(idx)
                .or_insert_with(HashSet::new)
                .insert(count);
        }
    }

    let filtered_counts: HashMap<String, usize> = counts.into_iter()
        .filter_map(|(idx, set)| {
            if set.len() >= 2 {
                Some((idx, set.len()))
            } else {
                None
            }
        })
        .collect();

    filtered_counts
}

// Returns a HashMap of name and index into outline
fn count_different_point_counts(masters: &Vec<UFO>) -> HashMap<String, HashMap<usize, HashSet<usize>>> {
    let point_count_set: Vec<HashMap<String, HashMap<usize, usize>>> = masters
        .iter()
        .map(|ufo| {
            ufo.glyph_entries
                .iter()
                .map(|ge| {
                    let point_counts: HashMap<usize, usize> = ge.glif.outline.clone()
                        .unwrap_or(Vec::new())
                        .iter()
                        .enumerate()
                        .map(|(i, contour)| (i, contour.len()))
                        .collect();

                    (ge.uniname.clone(), point_counts)
                })
                .collect()
        })
        .collect();

    let mut final_counts: HashMap<String, HashMap<usize, HashSet<usize>>> = HashMap::new();

    for count_set in point_count_set {
        for (glyph_name, count) in count_set {
            let glyph_count = final_counts.entry(glyph_name.clone()).or_insert(HashMap::new());

            for (i, point_len) in count.iter() {
                glyph_count
                    .entry(*i)
                    .or_insert(HashSet::new())
                    .insert(*point_len);
            }
        }
    }

    for (_, contour_map) in &mut final_counts {
        contour_map.retain(|_, v| v.len() > 1)
    }

    final_counts.retain(|_, v| v.len() > 0);
    final_counts
}

fn count_different_contour_open_state(masters: &Vec<UFO>) -> HashMap<String, HashMap<usize, HashSet<bool>>> {
    let point_count_set: Vec<HashMap<String, HashMap<usize, bool>>> = masters
        .iter()
        .map(|ufo| {
            ufo.glyph_entries
                .iter()
                .map(|ge| {
                    let point_counts: HashMap<usize, bool> = ge.glif.outline.clone()
                        .unwrap_or(Vec::new())
                        .iter()
                        .enumerate()
                        .map(|(i, contour)| (i, contour.is_open()))
                        .collect();

                    (ge.uniname.clone(), point_counts)
                })
                .collect()
        })
        .collect();

    let mut final_counts: HashMap<String, HashMap<usize, HashSet<bool>>> = HashMap::new();

    for count_set in point_count_set {
        for (glyph_name, count) in count_set {
            let glyph_count = final_counts.entry(glyph_name.clone()).or_insert(HashMap::new());

            for (i, point_len) in count.iter() {
                glyph_count
                    .entry(*i)
                    .or_insert(HashSet::new())
                    .insert(*point_len);
            }
        }
    }

    for (_, contour_map) in &mut final_counts {
        contour_map.retain(|_, v| v.len() > 1)
    }

    final_counts.retain(|_, v| v.len() > 0);
    final_counts
}
