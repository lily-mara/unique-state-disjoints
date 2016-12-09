extern crate scoped_threadpool;

use std::io::prelude::*;
use std::fs::File;
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver};
use scoped_threadpool::Pool;

type Charset<'a> = Box<[CharsetEntry<'a>]>;
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
}

fn merge_disjoints<'a>(disjoints: &CharsetComparisson<'a>) -> CharsetComparissonSync<'a> {
    let mut disjoint_vec = vec![];

    for disjoint in disjoints {
        disjoint_vec.push(disjoint);
    }

    disjoint_vec.sort_by(|x, y| x.state.original.cmp(y.state.original));
    disjoint_vec
}

fn find_unique_disjoints_async<'a>(states: &Charset<'a>,
                                   disjoints: &CharsetComparisson<'a>)
                                   -> CharsetComparisson<'a> {
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
                        }
                    }
                }
                if !fail {
                    match tx.send(Comparisson {
                        word: word_from_pair,
                        state: state_from_pair,
                    }) {
                        Ok(()) => {}
                        Err(e) => panic!("Failed to send between threads: {:?}", e),
                    }
                }
            });
        }
    });

    rx
}

fn find_disjoint_words_async<'a>(states: &'a Charset<'a>,
                                 words: &'a Charset<'a>)
                                 -> CharsetComparisson<'a> {
    let mut pool = Pool::new(POOL_SIZE);

    let (tx, rx) = channel();
    pool.scoped(|scope| {
        for state in states.iter() {
            let tx = tx.clone();
            scope.execute(move || {
                for word in words.iter() {
                    if word.chars.is_disjoint(&state.chars) {
                        match tx.send(Comparisson {
                            word: word,
                            state: state,
                        }) {
                            Ok(()) => {}
                            Err(e) => panic!("Failed to send between threads: {:?}", e),
                        }
                    }
                }
            });
        }
    });

    rx
}

fn generate_list_of_characters(words: &str) -> Charset {
    let mut word_list = vec![];

    for word in words.lines() {
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

    return word_list.into_boxed_slice();
}

fn open_word_list(filename: &str) -> String {
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
