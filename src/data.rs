use std::collections::HashMap;

pub fn get_common_dtu_courses() -> HashMap<&'static str, &'static str> {
    [
        // Mathematics and Computer Science
        ("01005", "Advanced Engineering Mathematics 1"),
        ("01006", "Advanced Engineering Mathematics 2"),
        ("01017", "Discrete Mathematics"),
        ("01035", "Mathematics 1"),
        ("01037", "Mathematics 2"),
        ("02101", "Introduction to Programming"),
        ("02102", "Algorithms and Data Structures"),
        ("02105", "Algorithms and Data Structures 2"),
        ("02110", "Algorithms and Data Structures"),
        ("02157", "Functional Programming"),
        ("02158", "Concurrent Programming"),
        ("02159", "Operating Systems"),
        ("02180", "Introduction to Artificial Intelligence"),
        ("02201", "Introduction to Database Systems"),
        ("02393", "Programming in C++"),
        ("02450", "Introduction to Machine Learning and Data Mining"),
        // Physics & Engineering
        ("10020", "Advanced Engineering Mathematics"),
        ("10333", "Solid Mechanics 1"),
        ("22100", "Electronics 1"),
        ("22101", "Electronics 2"),
        ("25100", "Introduction to Physics and Nanotechnology"),
        ("25200", "Classical Physics 1"),
        ("25201", "Classical Physics 2"),
        ("28000", "Introduction to Environmental Engineering"),
        ("31001", "Fluid Mechanics 1"),
        ("31002", "Fluid Mechanics 2"),
        // Add all your other courses here...
    ]
    .iter()
    .copied()
    .collect()
}
