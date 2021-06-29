use jieba_rs::Jieba;
use lazy_static::lazy_static;
use serde_json::json;
use std::collections::HashSet;
use std::convert::TryInto;
use std::iter::FromIterator;
use unicode_segmentation::UnicodeSegmentation;

pub fn get_words<'a>(text: &'a str, lang: &str) -> Vec<&'a str> {
    match lang {
        "en" => get_words_english(text),
        "zh" | "zh-CN" | "zh-TW" => get_words_chinese(text),
        _ => panic!("Got unsupported language for get_words: {}", text),
    }
}

fn get_words_english<'a>(text: &'a str) -> Vec<&'a str> {
    text.split_word_bounds().collect::<Vec<&str>>()
}

lazy_static! {
    static ref JIEBA: Jieba = Jieba::new();
}

fn get_words_chinese<'a>(text: &'a str) -> Vec<&'a str> {
    JIEBA.cut(text, false)
}

pub fn get_sentences<'a>(
    text: &'a str,
    words: &[&'a str],
    lang: &str,
) -> Option<(Vec<Vec<&'a str>>, Vec<i32>)> {
    // Only English sentence segmentation supported
    if lang != "en" {
        return None;
    }

    let mut sentence_stops: Vec<i32> = vec![0];

    let sentence_strs = text.split_sentence_bounds().collect::<Vec<&str>>();

    // Gets the indicies inside the words array of the ends of sentences
    // in the entire text. That is, the 0-indexed ith position of sentence_stops
    // will contain the index inside the words array that corresponds to
    // the exclusive end of the 1-indexed ith sentence. So sentence_stops[1]
    // points to the exclusive end of the first sentence inside the words array,
    // and we assume that sentence_stops[0], which is always 0, is the inclusive start
    // of the first sentence.
    let mut words_index = 0;
    for sentence in &sentence_strs {
        let mut length = sentence.len() as i32;
        while length > 0 {
            length -= words[words_index].len() as i32;
            words_index += 1;
        }
        sentence_stops.push(words_index.try_into().unwrap());
    }

    let mut sentence_arr: Vec<Vec<&str>> = vec![];
    let sentence_count = sentence_strs.len();

    for sentence_index in 1..(sentence_count + 1) {
        // the start index of sentence i (sentence_index) in the words array (inclusive)
        let start_index = sentence_stops[sentence_index - 1] as usize; // indicies are certainly positive, no problem converting
                                                                       // the end index of sentence i (sentence_index) in the words array (exclusive)
        let end_index = sentence_stops[sentence_index] as usize;

        let mut sentence: Vec<&str> = vec![];

        for word_index in start_index..end_index {
            sentence.push(words[word_index]);
        }

        sentence_arr.push(sentence);
    }

    Some((sentence_arr, sentence_stops))
}

lazy_static! {
    static ref STOP_CHARS: HashSet<&'static str> = HashSet::from_iter(
        "!\"#$%&'()*+,-./:;<=>?@[\\]^_{|}~`。？！，、；：“ ” ‘ ’「」『』（）【】—…-～	
《》〈〉_ "
            .split("")
    );
}

pub fn get_unique_words(words: &[&str]) -> (serde_json::Value, usize) {
    let mut unique_words = json!({});
    let mut total_word_count = 0usize;

    let map = match unique_words {
        serde_json::Value::Object(ref mut map) => map,
        _ => panic!("unique_words serde_json::Value isn't an Object!"),
    };

    for word in words.iter() {
        let lowercase = word.to_lowercase();
        if !STOP_CHARS.contains(&lowercase[..]) {
            match map.get(&lowercase) {
                Some(num_val) => {
                    let new_num = num_val.as_i64().unwrap() + 1i64;
                    map.insert(lowercase, json!(new_num));
                    total_word_count += 1;
                }
                None => {
                    map.insert(lowercase, json!(1));
                    total_word_count += 1;
                }
            };
        }
    }

    (unique_words, total_word_count)
}

pub fn get_or_query_string(
    string_opt: &Option<String>,
    lang_opt: &Option<String>,
) -> Option<String> {
    match string_opt {
        Some(ref string) => match lang_opt {
            Some(ref lang) => Some(
                get_words(&string[..], &lang[..])
                    .iter()
                    .filter_map(|&word| match word {
                        " " => None,
                        _ => Some(word),
                    })
                    .collect::<Vec<&str>>()
                    .join(" OR "),
            ),
            None => None,
        },
        None => None,
    }
}

const SMALL_PAGE_SIZE: i32 = 100;
const MEDIUM_PAGE_SIZE: i32 = 150;
const LARGE_PAGE_SIZE: i32 = 200;

pub fn get_pages<'a>(
    sentences_opt: &Option<(Vec<Vec<&'a str>>, Vec<i32>)>,
) -> (Vec<Vec<&'a str>>, Vec<Vec<&'a str>>, Vec<Vec<&'a str>>) {
    let mut pages_sm: Vec<Vec<&str>> = vec![];
    let mut pages_md: Vec<Vec<&str>> = vec![];
    let mut pages_lg: Vec<Vec<&str>> = vec![];

    if sentences_opt.is_none() {
        return (pages_sm, pages_md, pages_lg);
    }

    let sentences = &sentences_opt.as_ref().unwrap().0;

    let mut remain_pg_len_sm: i32 = SMALL_PAGE_SIZE;
    let mut curr_pg_sm = 0;
    pages_sm.push(vec![]);

    let mut remain_pg_len_md: i32 = MEDIUM_PAGE_SIZE;
    let mut curr_pg_md = 0;
    pages_md.push(vec![]);

    let mut remain_pg_len_lg: i32 = LARGE_PAGE_SIZE;
    let mut curr_pg_lg = 0;
    pages_lg.push(vec![]);

    for sentence in sentences {
        for word in sentence {
            pages_sm[curr_pg_sm].push(word);
            pages_md[curr_pg_md].push(word);
            pages_lg[curr_pg_lg].push(word);
        }

        let length: i32 = sentence.len().try_into().unwrap();
        remain_pg_len_sm -= length;
        remain_pg_len_md -= length;
        remain_pg_len_lg -= length;

        if remain_pg_len_sm <= 0 {
            remain_pg_len_sm = SMALL_PAGE_SIZE;
            curr_pg_sm += 1;
            pages_sm.push(vec![]);
        }

        if remain_pg_len_md <= 0 {
            remain_pg_len_md = MEDIUM_PAGE_SIZE;
            curr_pg_md += 1;
            pages_md.push(vec![]);
        }

        if remain_pg_len_lg <= 0 {
            remain_pg_len_lg = LARGE_PAGE_SIZE;
            curr_pg_lg += 1;
            pages_lg.push(vec![]);
        }
    }

    // remove potential empty last pages
    if pages_sm[pages_sm.len() - 1].is_empty() {
        pages_sm.pop();
    }

    if pages_md[pages_md.len() - 1].is_empty() {
        pages_md.pop();
    }

    if pages_lg[pages_lg.len() - 1].is_empty() {
        pages_lg.pop();
    }

    (pages_sm, pages_md, pages_lg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_word_split_1() {
        let words = get_words_english(
            r#"
            Chapter 1



            In my younger and more vulnerable years my father gave me some advice
            that I've been turning over in my mind ever since.
            
            "Whenever you feel like criticizing any one," he told me, "just
            remember that all the people in this world haven't had the advantages
            that you've had."
            "#,
        );

        assert_eq!(words[16], "and");
        assert_eq!(words[20], "vulnerable");
    }

    #[test]
    fn english_sentence_split_1() {
        let text = r#"
        Chapter 1
        In my younger and more vulnerable years. My father gave me some advice that I've been turning. Over in my mind ever since. "Whenever you feel like criticizing any one," he. Told me, "just remember that all the people in this world haven't had the advantages that you've had."
        "#;

        get_sentences(text, &get_words_english(text), "en");
    }
}
