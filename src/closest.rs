use strsim::levenshtein;

const MIN_DISTANCE: usize = 5;

pub fn find_closest_match<'a>(input: &str, list: &'a [&str]) -> Option<&'a str> {
    let mut closest_match = None;
    let mut min_distance = MIN_DISTANCE;

    for &item in list {
        let distance = levenshtein(input, item);
        if distance < min_distance {
            min_distance = distance;
            closest_match = Some(item);
        }
    }

    closest_match
}

#[test]
fn test() {
    let list = ["Hello", "Hi"];
    let input = "He";
    let closest = find_closest_match(input, &list);

    assert_eq!(closest, Some("Hi"))
}
