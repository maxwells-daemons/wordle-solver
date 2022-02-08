use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use hashbag::HashBag;
use std::fs::File;
use std::io::{self, BufRead, Write};

const WORD_LEN: usize = 5;
const NUM_BUCKETS: usize = usize::pow(3, WORD_LEN as u32) as usize; // 5 letters, 3 possibilities

type Word = [char; WORD_LEN]; // Stack-allocated fixed-size word for cache efficiency

// Optimal first word is always the same
const FIRST_GUESS: Word = ['r', 'a', 'i', 's', 'e'];

fn string_to_word(s: &str) -> Word {
    let mut word: Word = Default::default();
    for (i, c) in s.chars().enumerate() {
        word[i] = c;
    }
    word
}

fn word_to_string(word: &Word) -> String {
    word.iter().collect()
}

fn read_words(path: &str) -> Vec<Word> {
    let file = File::open(path).unwrap();
    io::BufReader::new(file)
        .lines()
        .map(|line| string_to_word(&line.unwrap()))
        .collect()
}

// Given a word and a pattern, find out which "information bucket" the pattern would match the word into.
// Each character position yields a trit, forming a trinary bucket index.
fn get_bucket(pattern: Word, answer: Word) -> usize {
    let mut bucket = 0;
    let mut letters: HashBag<char> = answer.into_iter().collect();

    for (p, w) in pattern.iter().zip(answer.iter()) {
        bucket *= 3; // Trinary SHL

        if p == w {
            bucket += 2; // Match-in-place: 2
            letters.remove(p);
        } else if letters.contains(p) > 0 {
            bucket += 1; // Match-out-of-place: 1
            letters.remove(p);
        } // No match: 0
    }

    bucket
}

fn bucketize_answers(answers: &Vec<Word>, pattern: Word) -> [Vec<Word>; NUM_BUCKETS] {
    const EMPTY_VEC: Vec<Word> = Vec::new();
    let mut buckets = [EMPTY_VEC; NUM_BUCKETS];
    for &answer in answers {
        let bucket = get_bucket(pattern, answer);
        buckets[bucket].push(answer);
    }
    buckets
}

fn bucket_counts(answers: &Vec<Word>, pattern: Word) -> [usize; NUM_BUCKETS] {
    let mut counts = [0; NUM_BUCKETS];
    for &answer in answers {
        let bucket = get_bucket(pattern, answer);
        counts[bucket] += 1;
    }
    counts
}

fn get_best_pattern(answers: &Vec<Word>, guesses: &Vec<Word>) -> Word {
    let mut best_pattern: Word = Default::default();
    let mut best_score = answers.len() + 1;

    for &pattern in guesses.iter().progress_with(
        ProgressBar::new(guesses.len() as u64).with_style(
            ProgressStyle::default_bar()
                .template("Finding pattern: [{elapsed} / {duration}] {wide_bar} {pos}/{len}"),
        ),
    ) {
        // The "score" of a pattern is the size of the largest bucket it splits
        // answers into; lower is better.
        let mut score = bucket_counts(answers, pattern).into_iter().max().unwrap();
        
        // Slightly prefer patterns that could also be an answer, in case we get lucky.
        // This helps break ties when there are only a few answers left.
        if answers.contains(&pattern) {
            score -= 1;
        }

        if score < best_score {
            best_pattern = pattern.clone();
            best_score = score;
            io::stdout().flush().unwrap();
        }
    }

    best_pattern
}

// + = match-in-place; - = match-out-of-place; . = no match
fn read_result() -> usize {
    print!("Enter result (+/-/.): ");
    io::stdout().flush().unwrap();
    let line = io::stdin().lock().lines().next().unwrap().unwrap();
    let mut bucket = 0;
    for c in line.chars() {
        bucket *= 3;
        match c {
            '+' => bucket += 2, // Match-in-place: 2
            '-' => bucket += 1, // Match-out-of-place: 1
            '.' => bucket += 0, // No match: 0
            _ => panic!("Invalid character"),
        }
    }
    bucket
}

fn main() {
    let mut answers = read_words("dictionaries/wordle.txt");
    let guesses = answers.clone();
    let mut pattern = FIRST_GUESS;

    loop {
        // User enters the selected pattern and sees a result
        println!("{} possible words", answers.len());
        println!("Enter pattern: {}", word_to_string(&pattern));
        let result = read_result();

        // Filter down answers to those that match the result
        let buckets = bucketize_answers(&answers, pattern);
        answers = buckets[result].clone();

        // If we've found an answer, we're done.
        // Otherwise, select a new pattern.
        if answers.is_empty() {
            println!("No words found");
            break;
        } else if answers.len() == 1 {
            println!("Found word: {}", word_to_string(&answers[0]));
            break;
        } else {
            pattern = get_best_pattern(&answers, &guesses);
        }
    }
}
