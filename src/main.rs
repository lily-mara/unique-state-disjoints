extern crate scoped_threadpool;
extern crate flame;

use std::io::prelude::*;
use std::fs::File;
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver};
use scoped_threadpool::Pool;

type CharsetComparisson<'a> = Receiver<Comparisson<'a>>;
type CharsetComparissonSync<'a> = Vec<Comparisson<'a>>;

const POOL_SIZE: u32 = 4;

struct Comparisson<'a> {
    word: &'a CharsetEntry<'a>,
    state: &'a CharsetEntry<'a>,
}

struct CharsetEntry<'a> {
    original: &'a str,
    chars: HashSet<char>,
}

fn main() {
    let _guard = flame::start_guard("main");
    let words = open_word_list("words.txt");
    let word_charset = generate_list_of_characters(&words);

    let states = open_word_list("states.txt");
    let state_charset = generate_list_of_characters(&states);

    let disjoints = find_disjoint_words_async(&state_charset, &word_charset);
    let unique_disjoints = find_unique_disjoints_async(&state_charset, &disjoints);

    let final_disjoints = merge_disjoints(&unique_disjoints);

    for word_pair in final_disjoints.iter() {
        let word = word_pair.word.original;
        let state = word_pair.state.original;
        println!("{} => {}", state, word);
    }
    flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap();
}

fn merge_disjoints<'a>(disjoints: &Receiver<Comparisson<'a>>) -> CharsetComparissonSync<'a> {
    let _guard = flame::start_guard("merge_disjoints");
    let mut disjoint_vec = vec![];

    for disjoint in disjoints {
        disjoint_vec.push(disjoint);
    }

    disjoint_vec.sort_by(|x, y| x.state.original.cmp(y.state.original));
    disjoint_vec
}

fn find_unique_disjoints_async<'a>(states: &'a [CharsetEntry<'a>],
                                   disjoints: &Receiver<Comparisson<'a>>)
                                   -> Receiver<Comparisson<'a>> {
    let _guard = flame::start_guard("find_unique_disjoints_async");
    let mut pool = Pool::new(POOL_SIZE);

    let (tx, rx) = channel();
    pool.scoped(|scope| {
        for word_pair in disjoints.iter() {
            let state_from_pair = word_pair.state;
            let word_from_pair = word_pair.word;
            let tx = tx.clone();
            scope.execute(move || {
                let mut fail = false;
                for state in states.iter() {
                    if state.original != state_from_pair.original {
                        if state.chars.is_disjoint(&word_from_pair.chars) {
                            fail = true;
                            break;
                        }
                    }
                }
                if !fail {
                    tx.send(Comparisson {
                            word: word_from_pair,
                            state: state_from_pair,
                        })
                        .expect("Failed to send between treads");
                }
            });
        }
    });

    rx
}

fn find_disjoint_words_async<'a>(states: &'a [CharsetEntry<'a>],
                                 words: &'a [CharsetEntry<'a>])
                                 -> CharsetComparisson<'a> {
    let _guard = flame::start_guard("find_disjoint_words_async");
    let mut pool = Pool::new(POOL_SIZE);

    let (tx, rx) = channel();
    pool.scoped(|scope| {
        for state in states.iter() {
            let tx = tx.clone();
            scope.execute(move || {
                for word in words.iter() {
                    if word.chars.is_disjoint(&state.chars) {
                        tx.send(Comparisson {
                                word: word,
                                state: state,
                            })
                            .expect("Failed to send between treads");
                    }
                }
            });
        }
    });

    rx
}

fn generate_list_of_characters<'a>(words: &'a str) -> Vec<CharsetEntry<'a>> {
    let _guard = flame::start_guard("generate_list_of_characters");
    let lines = words.lines();

    let mut word_list = match lines.size_hint() {
        (lower, Some(upper)) => Vec::with_capacity(upper - lower),
        _ => Vec::new(),
    };

    for word in lines {
        let mut chars = HashSet::new();
        for char in word.chars() {
            if char != ' ' {
                chars.insert(char);
            }
        }

        word_list.push(CharsetEntry {
            original: word,
            chars: chars,
        });
    }

    word_list
}

fn open_word_list(filename: &str) -> String {
    let _guard = flame::start_guard("open_word_list");
    let mut f = match File::open(filename) {
        Ok(f) => f,
        Err(_) => panic!("Unable to open wordlist!"),
    };

    let mut s = String::new();

    match f.read_to_string(&mut s) {
        Ok(_) => {}
        Err(_) => panic!("Unable to read from wordlist!"),
    };

    s
}
