#[derive(Debug)]
pub struct Querys {
    content: HashMap<String, String>,
}

impl PartialEq for Querys {
    fn eq(&self, other: &Self) -> bool {
        if self.content.len() != other.content.len() {
            // different number of entries is guaranteed different
            return false;
        }
        if !self.content.is_empty() {
            for key in self.content.keys() {
                if other.content.contains_key(key) {
                    if self.content.get(key).unwrap() != other.content.get(key).unwrap() {
                        // both have the same key, but associated value is different
                        return false;
                    };
                } else {
                    // missing one entry key makes them different
                    return false;
                };
            }
        };
        // both have the same length but are empty is guaranteed equal
        true
    }
}

impl Default for Querys {
    fn default() -> Self {
        Querys::new()
    }
}

impl Querys {
    #[must_use = "You wanted it, so take it!"]
    pub fn new() -> Self {
        Querys {
            content: HashMap::<String, String>::new(),
        }
    }

    #[must_use = "You wanted it, so take it!"]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.content.get(key)
    }

    /// # Errors
    ///
    /// Will return 'Error' if Query already contains a entry with given key.
    /// To avoid unintended behavior this method will return an Error.
    pub fn insert(&mut self, key: String, value: String) -> Result<(), Error> {
        if let Entry::Vacant(e) = self.content.entry(key) {
            e.insert(value);
            Ok(())
        } else {
            Err(Error::QueryKeyAlreadyExists)
        }
    }
}

#[cfg(test)]
mod querys_test {
    use super::{HashMap, Querys, TestCase};

    // Test Case with No Querys
    // both no query -> equal
    #[test]
    fn querys_ordering_empty_eq() {
        HashMap::<String, String>::new();
        let test = TestCase {
            case: Querys {
                content: HashMap::<String, String>::new(),
            },
            expected: Querys {
                content: HashMap::<String, String>::new(),
            },
        };
        assert_eq!(test.case, test.expected);
    }

    // Test Case Not Equal len
    // one with query and on without query -> not equal
    #[test]
    fn querys_ordering_empty_ne() {
        HashMap::<String, String>::new();
        let tests = [
            TestCase {
                case: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("bob"));
                        map
                    },
                },
                expected: Querys {
                    content: HashMap::<String, String>::new(),
                },
            },
            TestCase {
                expected: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("bob"));
                        map
                    },
                },
                case: Querys {
                    content: HashMap::<String, String>::new(),
                },
            },
        ];
        for test in tests.iter() {
            assert_ne!(test.case, test.expected);
        }
    }

    // Test Case with one Entrie
    // equal keys, equal values -> equal
    #[test]
    fn querys_ordering_one_eq() {
        let tests = [
            TestCase {
                case: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("bob"));
                        map
                    },
                },
                expected: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("bob"));
                        map
                    },
                },
            },
            TestCase {
                expected: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("bob"));
                        map
                    },
                },
                case: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("bob"));
                        map
                    },
                },
            },
        ];
        for test in tests.iter() {
            assert_eq!(test.case, test.expected);
        }
    }

    // Test Case with one Entrie
    // equal keys, not equal values -> not equal
    #[test]
    fn querys_ordering_one_ne() {
        let tests = [
            TestCase {
                case: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("peter"));
                        map
                    },
                },
                expected: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("bob"));
                        map
                    },
                },
            },
            TestCase {
                expected: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("age"), String::from("5"));
                        map
                    },
                },
                case: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("age"), String::from("10"));
                        map
                    },
                },
            },
        ];
        for test in tests.iter() {
            assert_ne!(test.case, test.expected);
        }
    }

    // Test Case with two Entries
    // equal keys, equal values -> equal
    #[test]
    fn querys_ordering_two_eq() {
        let tests = [
            TestCase {
                case: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("bob"));
                        map.insert(String::from("age"), String::from("10"));
                        map
                    },
                },
                expected: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("bob"));
                        map.insert(String::from("age"), String::from("10"));
                        map
                    },
                },
            },
            TestCase {
                expected: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("bob"));
                        map.insert(String::from("age"), String::from("10"));
                        map
                    },
                },
                case: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("bob"));
                        map.insert(String::from("age"), String::from("10"));
                        map
                    },
                },
            },
        ];
        for test in tests.iter() {
            assert_eq!(test.case, test.expected);
        }
    }

    // Test Case with two Entries
    // equal keys, not equal values -> not equal
    #[test]
    fn querys_ordering_two_ne() {
        let tests = [
            TestCase {
                case: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("bob"));
                        map.insert(String::from("age"), String::from("10"));
                        map
                    },
                },
                expected: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("peter"));
                        map.insert(String::from("age"), String::from("5"));
                        map
                    },
                },
            },
            TestCase {
                expected: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("peter"));
                        map.insert(String::from("age"), String::from("5"));
                        map
                    },
                },
                case: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("bob"));
                        map.insert(String::from("age"), String::from("10"));
                        map
                    },
                },
            },
        ];

        for test in tests.iter() {
            assert_ne!(test.case, test.expected);
        }
    }

    // Test Case with two Entries
    // unequal keys -> not equal
    #[test]
    fn querys_ordering_key_ne() {
        let tests = [
            TestCase {
                case: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("nama"), String::from("bob"));
                        map.insert(String::from("age"), String::from("10"));
                        map
                    },
                },
                expected: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("peter"));
                        map.insert(String::from("age"), String::from("5"));
                        map
                    },
                },
            },
            TestCase {
                expected: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("peter"));
                        map.insert(String::from("agu"), String::from("5"));
                        map
                    },
                },
                case: Querys {
                    content: {
                        let mut map = HashMap::<String, String>::new();
                        map.insert(String::from("name"), String::from("bob"));
                        map.insert(String::from("age"), String::from("10"));
                        map
                    },
                },
            },
        ];

        for test in tests.iter() {
            assert_ne!(test.case, test.expected);
        }
    }
}
