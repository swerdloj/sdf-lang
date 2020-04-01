// TODO: Arrays, then matrices
// Note that a matrix is just a 2d array of vecs/dvecs

// TODO: Just use a regex for this. Or implement a stack automata
pub fn is_mat_constructor_or_type(mut name: &str) -> bool {
    if name.len() > 0 && &name[0..=0] == "d" {
        name = &name[1..];
    }

    if !(name.len() > 3 && &name[0..=3] == "mat") {
        return false;
    }

    match name.len() {
        // matN
        4 => {
            match &name[3..=3] {
                "2" | "3" | "4" => true,
                _ => false,
            }
        }

        // matNxM
        6 => {
            match &name[3..=3] {
                "2" | "3" | "4" => {
                    if &name[4..=4] == "x" {
                        match &name[5..=5] {
                            "2" | "3" | "4" => true,
                            _ => false,
                        }
                    } else {
                        false
                    }
                },
                _ => false,
            }
        }

        _ => false,
    }
}

struct MatInputCursor<'types> {
    types: &'types Vec<String>,
    current: usize,
}

impl<'types> MatInputCursor<'types> {
    fn new(types: &'types Vec<String>) -> Self {
        MatInputCursor {
            types,
            current: 0,
        }
    }

    fn next_n(&mut self, n: usize) -> Vec<String> {
        let mut amount = 0;
        while amount < n {
            match self.types[self.current + amount].as_ref() {
                "vec2" | "dvec2" => {
                    amount += 2;
                }

                "int" | "float" | "double" => {
                    amount += 1;
                }

                _ => {
                    // TODO: Error
                }
            }
        }
        
        self.current += n;

        // TODO: Is this correct?
        self.types[(self.current - n) ..= self.current].to_vec()
    }
}

/// Returns mat type if the constructor is valid
pub fn validate_constructor(mut mat_type: &str, passed: &Vec<String>) -> Result<String, String> {
    let primitive = if &mat_type[0..=0] == "d" {
        "double"
    } else {
        "float"
    };

    let cursor = MatInputCursor::new(passed);

    // Matrix rows can be built just like vectors
    // can use vector validation to see if matrix rows are valid
    // then, if all rows are valid, the matrix is valid
    //
    // Valid:  mat3 z = mat3(vec2(1.), 1., vec3(1.), vec3(1.));

    match mat_type {
        // "mat2" | "dmat2" | "mat2x2" | "dmat2x2" => {
        //     if passed.len() != 
        //      ...
        //     vec::validate_constructor("vec2", cursor.next_n(2))
        // }

        _ => Err(format!("Unknown matrix type, '{}'", mat_type))
    }
}